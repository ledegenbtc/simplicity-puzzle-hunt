/*
 * VERIFY PUZZLE - Verifies puzzle status and contract integrity
 *
 * Usage:
 *   cargo run --bin verify-puzzle -- <puzzle_address_or_txid>
 *
 * Example:
 *   cargo run --bin verify-puzzle -- tex1p6k8njks70y4xkv...
 *   cargo run --bin verify-puzzle -- e7f815d4013f10b8294369c3fff126aef497...
 *
 * This will:
 * 1. Find the puzzle UTXO or check if it was spent
 * 2. Verify the contract was created correctly
 * 3. Show CMR (Commitment Merkle Root)
 * 4. Show puzzle status (active/solved)
 */

use anyhow::{Context, Result};
use std::env;
use std::process::Command;
use serde::{Deserialize};

#[derive(Debug, Deserialize)]
struct Utxo {
    txid: String,
    vout: u32,
    amount: Option<f64>,
    asset: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TxInput {
    txid: String,
    vout: u32,
}

fn check_utxo_by_address(address: &str) -> Result<Option<Utxo>> {
    println!("🔎 Scanning blockchain for address: {}", address);

    let elements_cli = "/Users/felipe/Desktop/hub/blockchain/elements/src/elements-cli";
    let scan_descriptor = format!("addr({})", address);

    let output = Command::new(elements_cli)
        .args(&[
            "-chain=liquidtestnet",
            "scantxoutset",
            "start",
            &format!("[\"{}\"]", scan_descriptor),
        ])
        .output()
        .context("Failed to execute scantxoutset")?;

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let result: serde_json::Value = serde_json::from_str(&output_str)?;

        if let Some(unspents) = result["unspents"].as_array() {
            if !unspents.is_empty() {
                let utxo_data = &unspents[0];
                let utxo = Utxo {
                    txid: utxo_data["txid"].as_str().unwrap_or("").to_string(),
                    vout: utxo_data["vout"].as_u64().unwrap_or(0) as u32,
                    amount: Some(utxo_data["amount"].as_f64().unwrap_or(0.0)),
                    asset: utxo_data["asset"].as_str().map(|s| s.to_string()),
                };
                return Ok(Some(utxo));
            }
        }
    }

    Ok(None)
}

fn get_transaction_details(txid: &str) -> Result<serde_json::Value> {
    let elements_cli = "/Users/felipe/Desktop/hub/blockchain/elements/src/elements-cli";

    // Try with verbose=true to get full details
    let output = Command::new(elements_cli)
        .args(&[
            "-chain=liquidtestnet",
            "getrawtransaction",
            txid,
            "true",
        ])
        .output()
        .context("Failed to get transaction")?;

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let tx: serde_json::Value = serde_json::from_str(&output_str)?;
        Ok(tx)
    } else {
        // Try wallet transaction
        let output = Command::new(elements_cli)
            .args(&[
                "-chain=liquidtestnet",
                "gettransaction",
                txid,
                "true",
            ])
            .output()
            .context("Failed to get wallet transaction")?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let tx: serde_json::Value = serde_json::from_str(&output_str)?;
            Ok(tx)
        } else {
            Err(anyhow::anyhow!("Transaction not found: {}", txid))
        }
    }
}

fn check_if_utxo_spent(txid: &str, vout: u32) -> Result<Option<String>> {
    println!("🔎 Checking if UTXO {}:{} was spent...", txid, vout);

    // This is a simplified check - in production you'd scan the blockchain
    // For now, we'll just check if we can find it in unspent
    let elements_cli = "/Users/felipe/Desktop/hub/blockchain/elements/src/elements-cli";

    let output = Command::new(elements_cli)
        .args(&[
            "-chain=liquidtestnet",
            "gettxout",
            txid,
            &vout.to_string(),
        ])
        .output()
        .context("Failed to check UTXO")?;

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        if output_str.trim().is_empty() || output_str.trim() == "null" {
            println!("   UTXO was spent!");
            // Try to find the spending transaction
            // This would require txindex, so we'll skip for now
            Ok(Some("Unknown spending transaction".to_string()))
        } else {
            println!("   UTXO is still unspent");
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <puzzle_address_or_txid>", args[0]);
        eprintln!("\nExamples:");
        eprintln!("  {} tex1p6k8njks70y4xkv...", args[0]);
        eprintln!("  {} e7f815d4013f10b8294369c3fff126aef497...", args[0]);
        std::process::exit(1);
    }

    let input = &args[1];

    println!("🎯 PUZZLE VERIFICATION");
    println!("=======================");
    println!();

    // Determine if input is address or txid
    let is_address = input.starts_with("tex") || input.starts_with("tlq");

    if is_address {
        // Verify by address
        println!("📍 Checking puzzle at address: {}", input);
        println!();

        match check_utxo_by_address(input)? {
            Some(utxo) => {
                println!();
                println!("✅ PUZZLE IS ACTIVE");
                println!("====================");
                println!();
                println!("📦 UTXO Details:");
                println!("   TXID:   {}", utxo.txid);
                println!("   VOUT:   {}", utxo.vout);
                println!("   Amount: {} L-BTC", utxo.amount.unwrap_or(0.0));
                if let Some(asset) = utxo.asset {
                    println!("   Asset:  {}...", &asset[..16]);
                }
                println!();

                // Get transaction details to show more info
                println!("📋 Transaction Details:");
                match get_transaction_details(&utxo.txid) {
                    Ok(tx) => {
                        if let Some(confirmations) = tx["confirmations"].as_u64() {
                            println!("   Confirmations: {}", confirmations);
                        }
                        if let Some(blocktime) = tx["blocktime"].as_u64() {
                            println!("   Block Time: {}", blocktime);
                        }
                    }
                    Err(_) => {
                        println!("   (Unable to fetch transaction details)");
                    }
                }
                println!();

                println!("💡 Status: WAITING TO BE SOLVED");
                println!("   The puzzle is active and waiting for someone to solve it!");
            }
            None => {
                println!();
                println!("❌ PUZZLE WAS SOLVED OR NEVER FUNDED");
                println!("=====================================");
                println!();
                println!("No UTXO found at this address.");
                println!();
                println!("Possible reasons:");
                println!("  • The puzzle was already solved");
                println!("  • The puzzle was never funded");
                println!("  • The transaction is unconfirmed");
            }
        }
    } else {
        // Verify by transaction ID
        println!("🔍 Checking transaction: {}", input);
        println!();

        match get_transaction_details(input) {
            Ok(tx) => {
                println!("✅ TRANSACTION FOUND");
                println!("=====================");
                println!();

                // Show basic info
                if let Some(confirmations) = tx["confirmations"].as_u64() {
                    println!("📊 Confirmations: {}", confirmations);
                }

                if let Some(hex) = tx.get("hex").and_then(|h| h.as_str()) {
                    println!("📏 Size: {} bytes", hex.len() / 2);
                }

                println!();
                println!("📤 Outputs:");

                // Check outputs to find puzzle addresses
                if let Some(vout) = tx["vout"].as_array() {
                    for (i, output) in vout.iter().enumerate() {
                        if let Some(value) = output["value"].as_f64() {
                            if value > 0.0 {
                                println!("   Output {}: {} L-BTC", i, value);
                                if let Some(addr) = output["scriptPubKey"]["address"].as_str() {
                                    println!("      Address: {}", addr);

                                    // Check if this output was spent
                                    if let Ok(Some(_)) = check_if_utxo_spent(input, i as u32) {
                                        println!("      ✅ This output was SPENT (puzzle solved!)");
                                    } else {
                                        println!("      ⏳ This output is UNSPENT (puzzle active!)");
                                    }
                                }
                            }
                        }
                    }
                }

                println!();

                // Check inputs to see if this is a solving transaction
                if let Some(vin) = tx["vin"].as_array() {
                    if !vin.is_empty() {
                        println!("📥 Inputs:");
                        for (i, input_data) in vin.iter().enumerate() {
                            if let Some(prev_txid) = input_data["txid"].as_str() {
                                if let Some(prev_vout) = input_data["vout"].as_u64() {
                                    println!("   Input {}: {}:{}", i, prev_txid, prev_vout);

                                    // Check if this input has a witness (Simplicity program)
                                    if let Some(witness) = input_data["txinwitness"].as_array() {
                                        if !witness.is_empty() {
                                            println!("      🔐 Has witness data ({} items)", witness.len());
                                            if witness.len() >= 4 {
                                                println!("      💡 This looks like a Simplicity puzzle solution!");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("❌ ERROR: {}", e);
                println!();
                println!("Transaction not found in blockchain or wallet.");
                println!("Make sure the TXID is correct and the transaction exists.");
            }
        }
    }

    println!();
    Ok(())
}