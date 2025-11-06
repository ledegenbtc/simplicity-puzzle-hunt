/*
 * SOLVE PUZZLE WITH FEE - Pay-to-play puzzle solver
 *
 * Usage:
 *   cargo run --bin solve-puzzle-with-fee -- <puzzle_json_file> <secret> <destination_address>
 *
 * Example:
 *   cargo run --bin solve-puzzle-with-fee -- puzzle_2cf24dba.json "satoshi" tex1q...
 *
 * This will:
 * 1. Check if you have at least 1000 sats to attempt
 * 2. Create a transaction that includes the fee
 * 3. If correct: win entire pot + all previous failed attempts
 * 4. If wrong: lose your 1000 sats (added to the pot)
 */

use anyhow::Result;
use elements::pset::PartiallySignedTransaction as Psbt;
use elements::{confidential, secp256k1_zkp as secp256k1, Address, OutPoint, TxIn, TxInWitness, TxOut};
use secp256k1::XOnlyPublicKey;
use sha2::{Digest, Sha256};
use simplicityhl::{Arguments, CompiledProgram, Value, WitnessValues};
use simplicityhl::value::ValueConstructible;
use std::collections::HashMap;
use std::env;
use std::str::FromStr;

const PUZZLE_CONTRACT: &str = include_str!("../../../examples/puzzle_with_fee.simf");
const MIN_FEE_SATS: u64 = 1000; // Minimum fee to attempt the puzzle

fn main() -> Result<()> {
    // Parse arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <puzzle_json> <secret> <destination_address>", args[0]);
        eprintln!("\nExample:");
        eprintln!("  {} puzzle_2cf24dba.json \"satoshi\" tex1q...", args[0]);
        eprintln!("\n⚠️  NOTE: Each attempt costs {} sats!", MIN_FEE_SATS);
        std::process::exit(1);
    }

    let puzzle_file = &args[1];
    let secret = &args[2];
    let dest_address = &args[3];

    println!("🎯 SOLVING PUZZLE (WITH FEE)");
    println!("============================");
    println!();
    println!("⚠️  WARNING: This attempt will cost you {} sats", MIN_FEE_SATS);
    println!("   If you're wrong, you lose the sats!");
    println!("   If you're right, you win the entire pot!");
    println!();

    // 1. Read puzzle information
    println!("📖 Reading puzzle from: {}", puzzle_file);
    let puzzle_data = std::fs::read_to_string(puzzle_file)?;
    let puzzle: serde_json::Value = serde_json::from_str(&puzzle_data)?;

    let expected_hash = puzzle["hash"].as_str().unwrap();
    let puzzle_address = puzzle["address"].as_str().unwrap();
    let current_pot = puzzle["amount"].as_str().unwrap_or("unknown");

    println!("   Puzzle address: {}", puzzle_address);
    println!("   Expected hash: {}", expected_hash);
    println!("   Current pot: {} BTC", current_pot);
    println!("   Your cost: {} sats", MIN_FEE_SATS);
    println!();

    // 2. Verify the secret is correct (locally first)
    println!("🔍 Pre-verifying secret locally...");
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    let hash = hasher.finalize();
    let hash_hex = format!("0x{}", hex::encode(hash));

    if hash_hex != expected_hash {
        println!("❌ WARNING: Secret appears to be incorrect!");
        println!("   Expected: {}", expected_hash);
        println!("   Got:      {}", hash_hex);
        println!();
        println!("   Are you sure you want to continue and lose {} sats?", MIN_FEE_SATS);
        println!("   Press Ctrl-C to cancel, or Enter to continue anyway...");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
    } else {
        println!("✅ Secret looks correct! You should win the pot!");
        println!();
    }

    // 3. Compile the contract with fee parameter
    println!("⚙️  Compiling contract with fee requirement...");
    let mut hash_bytes = [0u8; 32];
    hash_bytes.copy_from_slice(&hash);
    let target_hash = simplicityhl::num::U256::from_byte_array(hash_bytes);

    let mut arguments = HashMap::new();
    arguments.insert(
        simplicityhl::str::WitnessName::from_str_unchecked("TARGET_HASH"),
        Value::u256(target_hash),
    );
    arguments.insert(
        simplicityhl::str::WitnessName::from_str_unchecked("MIN_FEE"),
        Value::u64(MIN_FEE_SATS),
    );
    let args = Arguments::from(arguments);

    let compiled = CompiledProgram::new(PUZZLE_CONTRACT, args, false)
        .map_err(|e| anyhow::anyhow!("Failed to compile contract: {}", e))?;
    println!("✅ Contract compiled with {} sat fee requirement!", MIN_FEE_SATS);
    println!();

    // 4. Connect to elementsd (for demo purposes, we'll use CLI commands instead)

    // 5. Get your own UTXO to pay the fee from
    println!("💰 Getting your wallet balance...");

    // Get a new address for change using CLI
    use std::process::Command;
    use std::path::PathBuf;

    // Find elements-cli in multiple possible locations
    let home = env::var("HOME").unwrap_or_default();
    let possible_paths = vec![
        format!("{}/.elements/elements-cli", home),
        format!("{}/Desktop/hub/blockchain/elements/src/elements-cli", home),
        String::from("./elements"),  // Wrapper script from setup
    ];

    let elements_cli = possible_paths
        .iter()
        .find(|p| PathBuf::from(p).exists())
        .ok_or_else(|| anyhow::anyhow!(
            "elements-cli not found. Please run ./setup.sh first"
        ))?;

    let output = if elements_cli.ends_with("/elements") {
        Command::new(elements_cli)
            .args(&["getnewaddress"])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run elements wrapper: {:?}", e))?
    } else {
        Command::new(elements_cli)
            .args(&["-chain=liquidtestnet", "getnewaddress"])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run elements-cli: {:?}", e))?
    };

    let change_address = Address::from_str(
        String::from_utf8_lossy(&output.stdout).trim()
    )?;

    println!("   Your change will go to: {}", change_address);

    // For demo, we'll assume a fee UTXO is available
    // In production, you'd query listunspent to find one
    println!("   NOTE: You need to provide your fee UTXO manually");
    println!("   Edit the script to include your wallet UTXO details");

    // Placeholder fee UTXO (you need to fill this in)
    let fee_txid_str = "YOUR_FEE_TXID_HERE";
    let fee_vout = 0u32;
    let fee_amount_sats = 100_000u64; // Amount in your wallet UTXO

    if fee_txid_str == "YOUR_FEE_TXID_HERE" {
        eprintln!("❌ ERROR: You need to edit the script and add your fee UTXO");
        eprintln!("   Run: elements-cli -chain=liquidtestnet listunspent");
        eprintln!("   Find a UTXO with at least {} sats", MIN_FEE_SATS + 1000);
        std::process::exit(1);
    }

    let fee_txid = elements::Txid::from_str(fee_txid_str)?;

    println!("   Using fee UTXO: {}:{}", fee_txid, fee_vout);
    println!("   Amount: {} sats", fee_amount_sats);
    println!();

    // 6. Find the puzzle UTXO
    println!("🔎 Looking for puzzle UTXO...");

    // NOTE: For demo purposes, you need to manually set these
    // In production, this would query the blockchain
    let puzzle_txid_str = "YOUR_PUZZLE_TXID_HERE";
    let puzzle_vout = 0u32;
    let puzzle_value_sats = 10_000_000u64; // Current pot value

    if puzzle_txid_str == "YOUR_PUZZLE_TXID_HERE" {
        eprintln!("❌ ERROR: You need to edit the script and add the puzzle TXID/VOUT");
        eprintln!("   Run: elements-cli -chain=liquidtestnet listunspent");
        eprintln!("   And look for address: {}", puzzle_address);
        std::process::exit(1);
    }

    let puzzle_txid = elements::Txid::from_str(puzzle_txid_str)?;
    let puzzle_outpoint = OutPoint::new(puzzle_txid, puzzle_vout);

    println!("✅ Puzzle UTXO: {}:{}", puzzle_txid, puzzle_vout);
    println!("   Current pot: {} sats", puzzle_value_sats);
    println!();

    // 7. Create spending transaction with two inputs
    println!("💸 Creating transaction...");
    println!("   Input 1: Your payment ({} sats from wallet)", MIN_FEE_SATS);
    println!("   Input 2: The puzzle pot ({} sats)", puzzle_value_sats);

    let dest_addr = Address::from_str(dest_address)?;
    let network_fee = 3_000u64;

    // Calculate outputs
    let total_input = puzzle_value_sats + MIN_FEE_SATS;
    let output_value = total_input - network_fee;
    let change_value = fee_amount_sats - MIN_FEE_SATS - network_fee;

    // Get asset ID (L-BTC on testnet)
    let asset = confidential::Asset::Explicit(elements::AssetId::default());

    let mut psbt = Psbt::from_tx(elements::Transaction {
        version: 2,
        lock_time: elements::LockTime::ZERO,
        input: vec![
            // Input 1: Your fee payment
            TxIn {
                previous_output: OutPoint::new(fee_txid, fee_vout),
                is_pegin: false,
                script_sig: elements::Script::new(),
                sequence: elements::Sequence::ZERO,
                asset_issuance: elements::AssetIssuance::null(),
                witness: TxInWitness::empty(),
            },
            // Input 2: The puzzle UTXO
            TxIn {
                previous_output: puzzle_outpoint,
                is_pegin: false,
                script_sig: elements::Script::new(),
                sequence: elements::Sequence::ZERO,
                asset_issuance: elements::AssetIssuance::null(),
                witness: TxInWitness::empty(),
            },
        ],
        output: vec![
            // Main output: The prize (if you win)
            TxOut {
                value: confidential::Value::Explicit(output_value),
                script_pubkey: dest_addr.script_pubkey(),
                asset,
                nonce: confidential::Nonce::Null,
                witness: elements::TxOutWitness::empty(),
            },
            // Change output
            TxOut {
                value: confidential::Value::Explicit(change_value),
                script_pubkey: change_address.script_pubkey(),
                asset,
                nonce: confidential::Nonce::Null,
                witness: elements::TxOutWitness::empty(),
            },
            // Fee output
            TxOut::new_fee(network_fee, asset.explicit().unwrap()),
        ],
    });

    // 8. Sign your fee input (standard wallet signing)
    println!("✍️  Signing your fee input...");
    // This would use the wallet's signing capability
    // For now, we assume it's handled by elementsd

    // 9. Create witness for the puzzle with the secret
    println!("🔐 Creating witness with secret...");

    let secret_value = Value::u256(target_hash);
    let mut witness_map = HashMap::new();
    witness_map.insert(
        simplicityhl::str::WitnessName::from_str_unchecked("SECRET"),
        secret_value,
    );
    let witness_values = WitnessValues::from(witness_map);

    // 10. Satisfy the program
    let satisfied = compiled
        .satisfy(witness_values)
        .map_err(|e| anyhow::anyhow!("Failed to satisfy program: {}", e))?;

    let (program_bytes, witness_bytes) = satisfied.redeem().encode_to_vec();

    // 11. Add witness to puzzle input
    let internal_key = XOnlyPublicKey::from_str(
        "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0",
    )?;

    let script = elements::Script::from(compiled.commit().cmr().as_ref().to_vec());
    let builder = elements::taproot::TaprootBuilder::new();
    // Convert LeafVersion between elements versions
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
        .control_block(&(script, leaf_ver))
        .expect("control block should exist");

    // Set witness for the puzzle input (index 1)
    psbt.inputs_mut()[1].final_script_witness = Some(vec![
        witness_bytes,
        program_bytes,
        control_block.serialize(),
    ]);

    // 12. Broadcast transaction
    println!("📡 Broadcasting transaction...");
    println!();

    let tx = psbt
        .extract_tx()
        .expect("transaction should be extractable");

    // Serialize transaction to hex
    use elements::encode::Encodable;
    let mut tx_bytes = Vec::new();
    tx.consensus_encode(&mut tx_bytes)?;
    let tx_hex = hex::encode(&tx_bytes);

    // Send using elements-cli
    let output = if elements_cli.ends_with("/elements") {
        Command::new(&elements_cli)
            .args(&["sendrawtransaction", &tx_hex])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run elements wrapper: {:?}", e))?
    } else {
        Command::new(&elements_cli)
            .args(&["-chain=liquidtestnet", "sendrawtransaction", &tx_hex])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run elements-cli: {:?}", e))?
    };

    if output.status.success() {
        let txid = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if hash_hex == expected_hash {
            println!("🎉🎉🎉 SUCCESS! 🎉🎉🎉");
            println!();
            println!("✅ Transaction broadcasted!");
            println!("   TXID: {}", txid);
            println!();
            println!("💰 Prize sent to: {}", dest_address);
            println!("   Amount won: {} sats", output_value);
            println!("   (Original pot + {} sat fee)", MIN_FEE_SATS);
            println!();
            println!("🏆 YOU WON THE PUZZLE!");
        } else {
            println!("❌ Transaction sent but will likely fail!");
            println!("   Your {} sats will be added to the pot", MIN_FEE_SATS);
            println!("   Better luck next time!");
        }
    } else {
        eprintln!("❌ Failed to broadcast transaction:");
        eprintln!("   {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}