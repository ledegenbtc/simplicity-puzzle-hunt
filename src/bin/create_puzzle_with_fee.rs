/*
 * CREATE PUZZLE WITH FEE - Creates pay-to-play puzzles
 *
 * Usage:
 *   cargo run --bin create-puzzle-with-fee -- <secret> <initial_amount_btc>
 *
 * Example:
 *   cargo run --bin create-puzzle-with-fee -- "satoshi" 0.01
 *
 * This creates a puzzle where:
 * - Each guess costs 1000 sats
 * - Failed attempts add to the pot
 * - Winner takes all accumulated funds
 */

use anyhow::{Context, Result};
use elements::{confidential, secp256k1_zkp as secp256k1, Address};
use secp256k1::XOnlyPublicKey;
use sha2::{Digest, Sha256};
use simplicityhl::{Arguments, CompiledProgram, Value};
use simplicityhl::value::ValueConstructible;
use std::collections::HashMap;
use std::env;
use std::str::FromStr;

const PUZZLE_CONTRACT: &str = include_str!("../../../examples/puzzle_with_fee.simf");
const MIN_FEE_SATS: u64 = 1000;

fn main() -> Result<()> {
    // Parse arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <secret> <initial_amount_btc>", args[0]);
        eprintln!("\nExample:");
        eprintln!("  {} \"satoshi\" 0.01", args[0]);
        eprintln!("\nThis creates a pay-to-play puzzle where:");
        eprintln!("  - Each guess costs {} sats", MIN_FEE_SATS);
        eprintln!("  - Failed attempts grow the pot");
        eprintln!("  - Winner takes all!");
        std::process::exit(1);
    }

    let secret = &args[1];
    let amount_btc: f64 = args[2].parse()
        .map_err(|_| anyhow::anyhow!("Invalid amount"))?;

    if amount_btc < 0.00001 {
        eprintln!("❌ ERROR: Initial amount must be at least 0.00001 BTC");
        std::process::exit(1);
    }

    println!("🎯 CREATING PAY-TO-PLAY PUZZLE");
    println!("==============================");
    println!();
    println!("📝 Configuration:");
    println!("   Secret: \"{}\" (keep this safe!)", secret);
    println!("   Initial pot: {} BTC", amount_btc);
    println!("   Cost per guess: {} sats", MIN_FEE_SATS);
    println!();

    // 1. Calculate SHA256 hash of the secret
    println!("🔐 Hashing secret...");
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    let hash = hasher.finalize();
    let hash_hex = format!("0x{}", hex::encode(hash));

    println!("   Hash: {}", hash_hex);
    println!("   (Share this hash with participants)");
    println!();

    // 2. Compile the contract with parameters
    println!("⚙️  Compiling Simplicity contract...");

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

    let cmr = compiled.commit().cmr();
    println!("✅ Contract compiled!");
    println!("   CMR: {}", hex::encode(cmr.as_ref()));
    println!();

    // 3. Create Taproot address with the contract
    println!("🏦 Creating Taproot address...");

    // Use a provably unspendable internal key
    let internal_key = XOnlyPublicKey::from_str(
        "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0",
    )?;

    // Create script from CMR
    let script = elements::Script::from(cmr.as_ref().to_vec());

    // Build Taproot tree with our Simplicity script
    let builder = elements::taproot::TaprootBuilder::new();
    // Convert LeafVersion between elements versions
    let leaf_ver_inner: u8 = simplicity::leaf_version().into();
    let leaf_ver = elements::taproot::LeafVersion::from_u8(leaf_ver_inner)
        .expect("valid leaf version");
    let builder = builder
        .add_leaf_with_ver(0, script, leaf_ver)
        .expect("tap tree should be valid");

    let spend_info = builder
        .finalize(secp256k1::SECP256K1, internal_key)
        .expect("tap tree should be valid");

    // Create address for Liquid testnet
    use elements::AddressParams;
    let address = Address::p2tr(
        secp256k1::SECP256K1,
        internal_key,
        spend_info.merkle_root(),
        None,
        &AddressParams::LIQUID_TESTNET,
    );

    println!("✅ Puzzle address created!");
    println!("   Address: {}", address);
    println!();

    // 4. Connect to Elements daemon
    println!("🔌 Connecting to Elements daemon...");

    // 5. Fund the puzzle (we'll skip balance check for now)
    println!("💸 Funding puzzle with {} BTC...", amount_btc);

    let amount_sats = (amount_btc * 100_000_000.0) as u64;

    // Use raw RPC command to send transaction
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

    println!("   Using: {}", elements_cli);

    let output = if elements_cli.ends_with("/elements") {
        // Wrapper handles chain internally
        Command::new(elements_cli)
            .args(&[
                "sendtoaddress",
                &address.to_string(),
                &format!("{}", amount_btc),
            ])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run elements wrapper: {:?}", e))?
    } else {
        // Direct CLI needs chain argument
        Command::new(elements_cli)
            .args(&[
                "-chain=liquidtestnet",
                "sendtoaddress",
                &address.to_string(),
                &format!("{}", amount_btc),
            ])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run elements-cli: {:?}", e))?
    };

    if !output.status.success() {
        eprintln!("❌ ERROR: Failed to send transaction");
        eprintln!("   {}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }

    let tx = String::from_utf8_lossy(&output.stdout).trim().to_string();

    println!("✅ Puzzle funded!");
    println!("   TXID: {}", tx);
    println!();

    // 7. Save puzzle information to file
    let filename = format!("puzzle_fee_{}.json", &hash_hex[2..10]);
    let puzzle_data = serde_json::json!({
        "type": "pay_to_play",
        "secret": secret,
        "hash": hash_hex,
        "address": address.to_string(),
        "initial_amount": format!("{}", amount_btc),
        "min_fee_sats": MIN_FEE_SATS,
        "txid": tx,
        "created": chrono::Utc::now().to_rfc3339(),
    });

    std::fs::write(&filename, serde_json::to_string_pretty(&puzzle_data)?)?;

    // 8. Display summary
    println!("===========================================");
    println!("🎉 PAY-TO-PLAY PUZZLE CREATED!");
    println!("===========================================");
    println!();
    println!("📋 Share with participants:");
    println!("   Hash: {}", hash_hex);
    println!("   Address: {}", address);
    println!("   Initial pot: {} BTC", amount_btc);
    println!("   Cost per guess: {} sats", MIN_FEE_SATS);
    println!();
    println!("📑 Puzzle saved to: {}", filename);
    println!();
    println!("🎮 Game mechanics:");
    println!("   - Each wrong guess adds {} sats to the pot", MIN_FEE_SATS);
    println!("   - The pot grows with every failed attempt!");
    println!("   - First to guess correctly wins everything!");
    println!();
    println!("⚠️  IMPORTANT: Keep the secret \"{}\" safe!", secret);
    println!("   Share only the hash with participants!");
    println!();
    println!("💡 Hints you can share:");
    println!("   - The secret is case-sensitive");
    println!("   - Each attempt costs real money");
    println!("   - The pot is currently: {} BTC", amount_btc);
    println!();

    Ok(())
}