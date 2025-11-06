/*
 * EXPORT PROGRAM - Compiles puzzle contract and exports base64-encoded Simplicity program
 *
 * Usage:
 *   cargo run --bin export-program -- <secret>
 *
 * Example:
 *   cargo run --bin export-program -- "hello"
 */

use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha2::{Digest, Sha256};
use simplicityhl::{Arguments, CompiledProgram, Value};
use simplicityhl::value::ValueConstructible;
use std::collections::HashMap;
use std::env;

const PUZZLE_CONTRACT: &str = include_str!("../../../examples/puzzle_jackpot.simf");

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <secret>", args[0]);
        eprintln!("\nExample:");
        eprintln!("  {} \"hello\"", args[0]);
        std::process::exit(1);
    }

    let secret = &args[1];

    // Calculate hash of secret
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    let hash = hasher.finalize();
    let hash_hex = hex::encode(hash);

    // Convert to u256
    let mut hash_bytes = [0u8; 32];
    hash_bytes.copy_from_slice(&hash);
    let target_hash = simplicityhl::num::U256::from_byte_array(hash_bytes);

    // Compile the contract with the hash
    let mut arguments = HashMap::new();
    arguments.insert(
        simplicityhl::str::WitnessName::from_str_unchecked("TARGET_HASH"),
        Value::u256(target_hash),
    );
    let args = Arguments::from(arguments);

    let compiled = CompiledProgram::new(PUZZLE_CONTRACT, args, false)
        .map_err(|e| anyhow::anyhow!("Failed to compile: {}", e))?;

    // Get the program bytes and encode to base64
    let program_bytes = compiled.commit().to_vec_without_witness();
    let base64_program = STANDARD.encode(&program_bytes);

    // Print info
    eprintln!("Secret: {}", secret);
    eprintln!("Hash: 0x{}", hash_hex);
    eprintln!();
    eprintln!("Base64-encoded Simplicity program:");
    eprintln!();

    // Print the base64-encoded program
    println!("{}", base64_program);

    Ok(())
}
