/*
 * CHECK CONTRACT - Verifies Simplicity contract compilation and CMR
 *
 * Usage:
 *   cargo run --bin check-contract -- <secret_hash> [expected_address]
 *
 * Example:
 *   cargo run --bin check-contract -- 0x20f0e912902bfdc1ea47cdb5eadc6f5c1b3453f406f38dd34a92d0b30a270e22
 *   cargo run --bin check-contract -- 0x20f0e912... tex1p6k8njks70y4xkv...
 *
 * This will:
 * 1. Compile the Simplicity contract with the given hash
 * 2. Show the CMR (Commitment Merkle Root)
 * 3. Generate the taproot address
 * 4. Verify if it matches the expected address (if provided)
 */

use anyhow::{Context, Result};
use elements::{Address, AddressParams};
use elements::secp256k1_zkp as secp256k1;
use secp256k1::XOnlyPublicKey;
use simplicityhl::{Arguments, CompiledProgram, Value};
use simplicityhl::value::ValueConstructible;
use std::collections::HashMap;
use std::env;
use std::str::FromStr;

const PUZZLE_CONTRACT: &str = include_str!("../../../SimplicityHL/examples/puzzle_jackpot.simf");

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: {} <secret_hash> [expected_address]", args[0]);
        eprintln!("\nExamples:");
        eprintln!("  {} 0x20f0e912902bfdc1ea47cdb5eadc6f5c1b3453f406f38dd34a92d0b30a270e22", args[0]);
        eprintln!("  {} 0x20f0e912... tex1p6k8njks70y4xkv...", args[0]);
        std::process::exit(1);
    }

    let hash_str = &args[1];
    let expected_address = args.get(2);

    println!("🔧 CONTRACT VERIFICATION");
    println!("=========================");
    println!();

    // Parse hash
    let hash_hex = hash_str.trim_start_matches("0x");
    if hash_hex.len() != 64 {
        return Err(anyhow::anyhow!("Invalid hash length. Expected 64 hex characters (32 bytes)"));
    }

    let hash_bytes = hex::decode(hash_hex)
        .context("Failed to decode hash hex")?;

    if hash_bytes.len() != 32 {
        return Err(anyhow::anyhow!("Hash must be exactly 32 bytes"));
    }

    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&hash_bytes);
    let target_hash = simplicityhl::num::U256::from_byte_array(hash_array);

    println!("📝 Input:");
    println!("   Target Hash: 0x{}", hash_hex);
    println!();

    // Compile contract
    println!("⚙️  Compiling Simplicity contract...");
    let mut arguments = HashMap::new();
    arguments.insert(
        simplicityhl::str::WitnessName::from_str_unchecked("TARGET_HASH"),
        Value::u256(target_hash),
    );
    let args_compiled = Arguments::from(arguments);

    let compiled = CompiledProgram::new(PUZZLE_CONTRACT, args_compiled, false)
        .map_err(|e| anyhow::anyhow!("Failed to compile contract: {}", e))?;

    println!("✅ Contract compiled successfully!");
    println!();

    // Get CMR (Commitment Merkle Root)
    let cmr = compiled.commit().cmr();
    let cmr_hex = hex::encode(cmr.as_ref());

    println!("🔐 Contract Details:");
    println!("   CMR (Commitment Merkle Root):");
    println!("   0x{}", cmr_hex);
    println!("   Length: {} bytes", cmr.as_ref().len());
    println!();

    // Create taproot address
    println!("🏗️  Building Taproot Address...");

    let internal_key = XOnlyPublicKey::from_str(
        "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0",
    )?;

    let script = elements::Script::from(cmr.as_ref().to_vec());
    let builder = elements::taproot::TaprootBuilder::new();

    let leaf_ver_inner: u8 = simplicity::leaf_version().into();
    let leaf_ver = elements::taproot::LeafVersion::from_u8(leaf_ver_inner)
        .expect("valid leaf version");

    let builder = builder
        .add_leaf_with_ver(0, script, leaf_ver)
        .expect("tap tree should be valid");

    let spend_info = builder
        .finalize(secp256k1::SECP256K1, internal_key)
        .expect("tap tree should be valid");

    let address = Address::p2tr(
        secp256k1::SECP256K1,
        spend_info.internal_key(),
        spend_info.merkle_root(),
        None,
        &AddressParams::LIQUID_TESTNET,
    );

    println!("✅ Taproot Structure:");
    println!("   Internal Key: {}", internal_key);
    if let Some(merkle_root) = spend_info.merkle_root() {
        println!("   Merkle Root:  {}", merkle_root);
    }
    println!();

    println!("📍 Generated Address:");
    println!("   {}", address);
    println!();

    // Verify if matches expected address
    if let Some(expected) = expected_address {
        println!("🔍 Verification:");
        if address.to_string() == *expected {
            println!("   ✅ SUCCESS! Address matches expected address!");
            println!();
            println!("   The contract was compiled correctly and the");
            println!("   puzzle was created with the right parameters.");
        } else {
            println!("   ❌ MISMATCH! Addresses do not match!");
            println!();
            println!("   Expected: {}", expected);
            println!("   Got:      {}", address);
            println!();
            println!("   This means either:");
            println!("   • The hash used is different");
            println!("   • The contract code was modified");
            println!("   • The internal key is different");
        }
    } else {
        println!("💡 Use this command with the expected address to verify:");
        println!("   cargo run --bin check-contract -- 0x{} {}", hash_hex, address);
    }

    println!();
    println!("📊 Summary:");
    println!("   Contract compilation: ✅ OK");
    println!("   CMR generation:       ✅ OK");
    println!("   Address generation:   ✅ OK");

    if expected_address.is_some() {
        if address.to_string() == *expected_address.unwrap() {
            println!("   Address verification: ✅ MATCH");
        } else {
            println!("   Address verification: ❌ MISMATCH");
        }
    }

    println!();

    Ok(())
}