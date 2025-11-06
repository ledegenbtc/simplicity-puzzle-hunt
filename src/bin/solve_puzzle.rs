/*
 * SOLVE PUZZLE - Solves and claims the prize from a puzzle
 *
 * Usage:
 *   cargo run --bin solve-puzzle -- <puzzle_json_file> <secret> <destination_address>
 *
 * Example:
 *   cargo run --bin solve-puzzle -- puzzle_2cf24dba.json "satoshi" tex1q...
 *
 * This will:
 * 1. Read the puzzle information
 * 2. Automatically find the puzzle UTXO
 * 3. Create a transaction spending the puzzle UTXO
 * 4. Provide the secret as witness
 * 5. Broadcast and win the prize!
 */

use anyhow::{Context, Result};
use elements::pset::PartiallySignedTransaction as Psbt;
use elements::{confidential, secp256k1_zkp as secp256k1, Address, OutPoint, TxIn, TxInWitness, TxOut};
use secp256k1::XOnlyPublicKey;
use sha2::{Digest, Sha256};
use simplicityhl::{Arguments, CompiledProgram, Value, WitnessValues};
use simplicityhl::value::ValueConstructible;
use std::collections::HashMap;
use std::env;
use std::process::Command;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

const PUZZLE_CONTRACT: &str = include_str!("../../../SimplicityHL/examples/puzzle_jackpot.simf");

#[derive(Debug, Deserialize, Serialize)]
struct PuzzleInfo {
    secret: String,
    hash: String,
    address: String,
    amount: String,
    hint: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Utxo {
    txid: String,
    vout: u32,
    #[serde(default)]
    amount: Option<f64>,
    #[serde(default)]
    value: Option<f64>,
    asset: Option<String>,
}

fn find_puzzle_utxo(address: &str) -> Result<Option<Utxo>> {
    println!("🔎 Searching for UTXOs at address: {}", address);

    let elements_cli = "/Users/felipe/Desktop/hub/blockchain/elements/src/elements-cli";

    // Try scantxoutset to find UTXOs for any address (not just wallet addresses)
    let scan_descriptor = format!("addr({})", address);

    println!("   Starting blockchain scan (this may take a moment)...");
    let output = Command::new(elements_cli)
        .args(&[
            "-chain=liquidtestnet",
            "scantxoutset",
            "start",
            &format!("[\"{}\"]", scan_descriptor),
        ])
        .output()
        .context("Failed to execute elements-cli scantxoutset")?;

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let result: serde_json::Value = serde_json::from_str(&output_str)
            .context("Failed to parse scantxoutset output")?;

        if let Some(unspents) = result["unspents"].as_array() {
            if !unspents.is_empty() {
                println!("   Found {} UTXO(s)", unspents.len());

                let utxo_data = &unspents[0];
                let utxo = Utxo {
                    txid: utxo_data["txid"].as_str().unwrap_or("").to_string(),
                    vout: utxo_data["vout"].as_u64().unwrap_or(0) as u32,
                    amount: Some(utxo_data["amount"].as_f64().unwrap_or(0.0)),
                    value: None,
                    asset: utxo_data["asset"].as_str().map(|s| s.to_string()),
                };
                return Ok(Some(utxo));
            }
        }
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        println!("   Warning: scantxoutset failed: {}", error);
    }

    println!("   No UTXOs found at address");
    Ok(None)
}

fn get_asset_id() -> Result<String> {
    // Get L-BTC asset ID for testnet
    let elements_cli = "/Users/felipe/Desktop/hub/blockchain/elements/src/elements-cli";

    let output = Command::new(elements_cli)
        .args(&[
            "-chain=liquidtestnet",
            "dumpassetlabels",
        ])
        .output()
        .context("Failed to get asset labels")?;

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let labels: serde_json::Value = serde_json::from_str(&output_str)?;

        // Look for bitcoin asset
        for (asset_id, label) in labels.as_object().unwrap() {
            if label.as_str().unwrap_or("").to_lowercase().contains("bitcoin") {
                return Ok(asset_id.clone());
            }
        }
    }

    // Default L-BTC asset ID for Liquid testnet
    Ok("144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49".to_string())
}

fn main() -> Result<()> {
    // Parse arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <puzzle_json> <secret> <destination_address>", args[0]);
        eprintln!("\nExample:");
        eprintln!("  {} puzzle_2cf24dba.json \"satoshi\" tex1q...", args[0]);
        std::process::exit(1);
    }

    let puzzle_file = &args[1];
    let secret = &args[2];
    let dest_address = &args[3];

    println!("🎯 SOLVING PUZZLE");
    println!("==================");
    println!();

    // 1. Read puzzle information
    println!("📖 Reading puzzle from: {}", puzzle_file);
    let puzzle_data = std::fs::read_to_string(puzzle_file)?;
    let puzzle: PuzzleInfo = serde_json::from_str(&puzzle_data)?;

    println!("   📍 Puzzle address: {}", puzzle.address);
    println!("   🔐 Expected hash: {}", puzzle.hash);
    println!("   💰 Prize amount: {} L-BTC", puzzle.amount);
    println!("   💡 Hint: {}", puzzle.hint);
    println!();

    // 2. Verify the secret is correct
    println!("🔍 Verifying secret...");

    // Convert secret to u256 (32 bytes with right-padding) to match Simplicity contract
    let mut secret_bytes = [0u8; 32];
    let secret_raw = secret.as_bytes();
    let len = secret_raw.len().min(32);
    secret_bytes[32 - len..].copy_from_slice(&secret_raw[..len]);

    // Calculate SHA256 of the padded secret
    let mut hasher = Sha256::new();
    hasher.update(&secret_bytes);
    let hash = hasher.finalize();
    let hash_hex = format!("0x{}", hex::encode(hash));

    if hash_hex != puzzle.hash {
        eprintln!("❌ ERROR: Incorrect secret!");
        eprintln!("   Expected hash: {}", puzzle.hash);
        eprintln!("   Your hash:     {}", hash_hex);
        eprintln!("   Your secret:   \"{}\"", secret);
        eprintln!();
        eprintln!("💡 Hint: {}", puzzle.hint);
        std::process::exit(1);
    }

    println!("✅ Secret is correct!");
    println!();

    // 3. Find the puzzle UTXO
    println!("🔎 Looking for puzzle UTXO...");

    let utxo = find_puzzle_utxo(&puzzle.address)?
        .ok_or_else(|| anyhow::anyhow!(
            "No UTXO found for puzzle address: {}\n\
             Possible reasons:\n\
             - The puzzle has already been solved\n\
             - The puzzle hasn't been funded yet\n\
             - The transaction is still unconfirmed",
            puzzle.address
        ))?;

    println!("✅ Found UTXO!");
    println!("   TXID: {}", utxo.txid);
    println!("   VOUT: {}", utxo.vout);

    let amount_btc = utxo.amount.or(utxo.value).unwrap_or(0.0);
    let value_sats = (amount_btc * 100_000_000.0) as u64;
    println!("   Amount: {} L-BTC ({} sats)", amount_btc, value_sats);
    println!();

    // 4. Compile the contract
    println!("⚙️  Compiling Simplicity contract...");
    let mut hash_bytes = [0u8; 32];
    hash_bytes.copy_from_slice(&hash);
    let target_hash = simplicityhl::num::U256::from_byte_array(hash_bytes);

    let mut arguments = HashMap::new();
    arguments.insert(
        simplicityhl::str::WitnessName::from_str_unchecked("TARGET_HASH"),
        Value::u256(target_hash),
    );
    let args = Arguments::from(arguments);

    let compiled = CompiledProgram::new(PUZZLE_CONTRACT, args, false)
        .map_err(|e| anyhow::anyhow!("Failed to compile contract: {}", e))?;
    println!("✅ Contract compiled!");
    println!();

    // 5. Get asset ID
    let asset_id_str = utxo.asset.clone()
        .unwrap_or_else(|| {
            println!("⚠️  No asset ID in UTXO, using default L-BTC asset ID");
            get_asset_id().unwrap_or_else(|_| {
                "144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49".to_string()
            })
        });

    println!("🪙 Asset ID: {}", &asset_id_str[..8]);
    println!();

    // 6. Create spending transaction
    println!("💸 Creating spending transaction...");

    let dest_addr = Address::from_str(dest_address)?;
    let fee_sats = 3_000u64;

    if value_sats <= fee_sats {
        return Err(anyhow::anyhow!(
            "UTXO value ({} sats) is too small to pay fee ({} sats)",
            value_sats, fee_sats
        ));
    }

    let output_value = value_sats - fee_sats;
    println!("   Output: {} sats", output_value);
    println!("   Fee:    {} sats", fee_sats);
    println!("   To:     {}", dest_address);
    println!();

    let txid = elements::Txid::from_str(&utxo.txid)?;
    let outpoint = OutPoint::new(txid, utxo.vout);

    let asset_id = elements::AssetId::from_str(&asset_id_str)?;
    let asset = confidential::Asset::Explicit(asset_id);

    let mut psbt = Psbt::from_tx(elements::Transaction {
        version: 2,
        lock_time: elements::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: outpoint,
            is_pegin: false,
            script_sig: elements::Script::new(),
            sequence: elements::Sequence::ZERO,
            asset_issuance: elements::AssetIssuance::null(),
            witness: TxInWitness::empty(),
        }],
        output: vec![
            TxOut {
                value: confidential::Value::Explicit(output_value),
                script_pubkey: dest_addr.script_pubkey(),
                asset,
                nonce: confidential::Nonce::Null,
                witness: elements::TxOutWitness::empty(),
            },
            TxOut::new_fee(fee_sats, asset.explicit().unwrap()),
        ],
    });

    // 7. Create witness with the secret
    println!("🔐 Creating witness with secret...");

    // Use the same secret_bytes we calculated earlier
    let secret_u256 = simplicityhl::num::U256::from_byte_array(secret_bytes);

    let mut witness_map = HashMap::new();
    witness_map.insert(
        simplicityhl::str::WitnessName::from_str_unchecked("SECRET"),
        Value::u256(secret_u256),
    );
    let witness_values = WitnessValues::from(witness_map);

    // 8. Satisfy the program and create final witness
    println!("🔓 Satisfying Simplicity program...");
    let satisfied = compiled
        .satisfy(witness_values)
        .map_err(|e| anyhow::anyhow!("Failed to satisfy program: {}", e))?;

    let (program_bytes, witness_bytes) = satisfied.redeem().to_vec_with_witness();
    println!("   Program size: {} bytes", program_bytes.len());
    println!("   Witness size: {} bytes", witness_bytes.len());
    println!();

    // 9. Add witness to transaction
    println!("🔧 Building taproot witness...");
    let internal_key = XOnlyPublicKey::from_str(
        "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0",
    )?;

    let script = elements::Script::from(compiled.commit().cmr().as_ref().to_vec());
    let builder = elements::taproot::TaprootBuilder::new();

    let leaf_ver_inner: u8 = simplicity::leaf_version().into();
    let leaf_ver = elements::taproot::LeafVersion::from_u8(leaf_ver_inner)
        .expect("valid leaf version");

    let builder = builder
        .add_leaf_with_ver(0, script.clone(), leaf_ver)
        .expect("tap tree should be valid");

    let spend_info = builder
        .finalize(secp256k1::SECP256K1, internal_key)
        .expect("tap tree should be valid");

    let control_block = spend_info
        .control_block(&(script.clone(), leaf_ver))
        .expect("control block should exist");

    // Build the final witness
    let witness_stack = vec![
        witness_bytes,
        program_bytes,
        script.as_bytes().to_vec(),
        control_block.serialize(),
    ];

    let mut tx = psbt.extract_tx()
        .map_err(|e| anyhow::anyhow!("Failed to extract transaction: {:?}", e))?;
    tx.input[0].witness = TxInWitness {
        script_witness: witness_stack,
        pegin_witness: vec![],
        amount_rangeproof: None,
        inflation_keys_rangeproof: None,
    };

    // 10. Broadcast transaction
    println!("📡 Broadcasting transaction...");
    println!("   Transaction size: {} bytes", elements::encode::serialize(&tx).len());

    let tx_hex = hex::encode(elements::encode::serialize(&tx));

    let elements_cli = "/Users/felipe/Desktop/hub/blockchain/elements/src/elements-cli";
    let output = Command::new(elements_cli)
        .args(&[
            "-chain=liquidtestnet",
            "sendrawtransaction",
            &tx_hex,
        ])
        .output()
        .context("Failed to broadcast transaction")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to broadcast transaction: {}", error));
    }

    let broadcast_txid = String::from_utf8_lossy(&output.stdout).trim().to_string();

    println!();
    println!("🎉🎉🎉 SUCCESS! 🎉🎉🎉");
    println!();
    println!("✅ Transaction broadcasted!");
    println!("   TXID: {}", broadcast_txid);
    println!();
    println!("💰 Prize sent to: {}", dest_address);
    println!("   Amount: {} sats (~{} L-BTC)", output_value, output_value as f64 / 100_000_000.0);
    println!();
    println!("🏆 YOU WON THE PUZZLE!");
    println!();
    println!("📊 Check your transaction:");
    println!("   elements-cli gettransaction {}", broadcast_txid);

    Ok(())
}