# How to Verify the Contract Matches

This guide shows you how to verify that the compiled Simplicity contract matches what's actually on-chain.

## Method 1: Verify CMR Consistency

The CMR (Commitment Merkle Root) should be identical for the same secret.

### Step 1: Export and analyze your contract

```bash
# For the "hello" puzzle
cargo run --bin export-program -- "hello" 2>/dev/null | \
  hal-simplicity simplicity simplicity info "$(cat)" | \
  jq -r '.cmr'
```

Expected output: `88090352fc40bf24ec6b607b5b56bf7d2f9e47676223f2359fe9fda1cd448194`

### Step 2: Verify it's consistent

Run it multiple times - the CMR should always be identical for the same secret.

```bash
# Try it 5 times
for i in {1..5}; do
  cargo run --bin export-program -- "hello" 2>/dev/null | \
    hal-simplicity simplicity simplicity info "$(cat)" | \
    jq -r '.cmr'
done
```

All outputs should be identical!

## Method 2: Verify by Attempting to Solve

The ultimate verification is to actually solve the puzzle and claim the funds.

### Step 1: Get UTXO information

```bash
cd $HOME/Desktop/hub/blockchain/elements

# Get UTXOs for the puzzle address
./src/elements-cli -chain=liquidtestnet listunspent 0 9999999 \
  '["tex1pd77euywmg452m55mpfj0c5g434csl3ug8nl9y7k9gyc2fgh6xwfqdzyh7a"]'
```

This will show you:
- `txid` - Transaction ID that funded the puzzle
- `vout` - Output index
- `amount` - Amount in BTC
- `assetcommitment` - The asset ID

### Step 2: Create a destination address

```bash
# Get a new address to receive the funds
./src/elements-cli -chain=liquidtestnet getnewaddress
```

### Step 3: Update solve_puzzle.rs with UTXO details

Edit `src/bin/solve_puzzle.rs` around line 120:

```rust
// UTXO information from listunspent
let txid_str = "YOUR_TXID_HERE";
let vout = 0; // from listunspent
let value_sats = 10000; // amount in satoshis (0.0001 BTC = 10000 sats)
```

### Step 4: Attempt to solve

```bash
cargo run --bin solve-puzzle -- puzzle_2cf24dba.json "hello" <your_address>
```

If the contract matches:
- ✅ Transaction will be accepted by the network
- ✅ Funds will be transferred to your address
- ✅ Contract is verified!

If it doesn't match:
- ❌ Transaction will be rejected
- ❌ You'll get an error message

## Method 3: Compare Transaction Scripts

### Step 1: Get the funding transaction

```bash
cd $HOME/Desktop/hub/blockchain/elements

# Get the transaction that funded the puzzle
./src/elements-cli -chain=liquidtestnet getrawtransaction <txid> true
```

### Step 2: Extract the scriptPubKey

Look for the output with your puzzle address and find its `scriptPubKey` field.

For a Taproot address, it should look like:
```json
{
  "scriptPubKey": {
    "asm": "1 <32-byte-taproot-output>",
    "hex": "5120<64-hex-chars>",
    "address": "tex1pd77euy...",
    "type": "witness_v1_taproot"
  }
}
```

### Step 3: Decode the Taproot output

The 32-byte value in the scriptPubKey is derived from:
- Internal key (fixed for all puzzles)
- Merkle root of the script tree (contains the Simplicity contract CMR)

## Method 4: Test with a Wrong Secret

Negative testing - try solving with the wrong secret to verify rejection.

```bash
# This should FAIL
cargo run --bin solve-puzzle -- puzzle_2cf24dba.json "wrong_secret" <address>
```

Expected behavior:
- Transaction should be rejected
- Error message about script verification failure

This confirms the contract is checking the secret!

## Method 5: Binary Comparison

Compare the exact bytecode from two compilations.

```bash
# Compile twice and compare
cargo run --bin export-program -- "hello" 2>/dev/null > /tmp/compile1.b64
cargo run --bin export-program -- "hello" 2>/dev/null > /tmp/compile2.b64

# Should be identical
diff /tmp/compile1.b64 /tmp/compile2.b64
```

No output = files are identical = deterministic compilation!

## Method 6: Full Round-Trip Test

Create a new puzzle and solve it immediately to verify the entire flow.

### Step 1: Create a test puzzle

```bash
cargo run --bin create-puzzle -- "test_secret_$(date +%s)" 0.0001
```

This creates a new puzzle with a timestamp-based secret.

### Step 2: Get the UTXO info

```bash
# Wait for confirmation, then list UTXOs
./elements-cli.sh listunspent 0 9999999 '["<new_puzzle_address>"]'
```

### Step 3: Solve it immediately

Update `solve_puzzle.rs` with the UTXO info and run:

```bash
cargo run --bin solve-puzzle -- puzzle_<hash>.json "<your_test_secret>" <dest_address>
```

If everything matches:
- ✅ You just created and solved your own puzzle
- ✅ Contract compilation is verified
- ✅ Full flow works!

## Understanding Address Differences

You might notice the address from `hal-simplicity` differs from the actual puzzle address:

**hal-simplicity address:**
```
tex1prtk6va387h5kuvag7hwm2wjuzrwlutta4ghj489xglnrvvsr30hs3l88j4
```

**Actual puzzle address:**
```
tex1pd77euywmg452m55mpfj0c5g434csl3ug8nl9y7k9gyc2fgh6xwfqdzyh7a
```

**Why?** Different Taproot constructions:
- `hal-simplicity` derives address from CMR alone (unconfined)
- `create-puzzle` uses a specific internal key + CMR

Both are valid! The CMR is what matters for contract verification.

## What to Verify

### ✅ Should be identical:
- CMR for same secret
- Base64 program encoding
- Program structure from hal-simplicity
- Binary bytecode

### ❌ Won't match:
- Final Taproot address (uses different internal key)
- Witness data (not part of commitment)

## Quick Verification Script

Create `verify.sh`:

```bash
#!/bin/bash

SECRET="$1"

echo "=== VERIFYING CONTRACT FOR SECRET: $SECRET ==="
echo

# Compile 3 times
echo "Compiling 3 times..."
CMR1=$(cargo run --bin export-program -- "$SECRET" 2>/dev/null | \
       hal-simplicity simplicity simplicity info "$(cat)" 2>/dev/null | \
       jq -r '.cmr')
CMR2=$(cargo run --bin export-program -- "$SECRET" 2>/dev/null | \
       hal-simplicity simplicity simplicity info "$(cat)" 2>/dev/null | \
       jq -r '.cmr')
CMR3=$(cargo run --bin export-program -- "$SECRET" 2>/dev/null | \
       hal-simplicity simplicity simplicity info "$(cat)" 2>/dev/null | \
       jq -r '.cmr')

echo "CMR #1: $CMR1"
echo "CMR #2: $CMR2"
echo "CMR #3: $CMR3"
echo

if [ "$CMR1" = "$CMR2" ] && [ "$CMR2" = "$CMR3" ]; then
    echo "✅ VERIFIED: CMR is deterministic"
else
    echo "❌ FAILED: CMRs don't match!"
    exit 1
fi

# Show program structure
echo
echo "=== CONTRACT STRUCTURE ==="
cargo run --bin export-program -- "$SECRET" 2>/dev/null | \
  hal-simplicity simplicity simplicity info "$(cat)" 2>/dev/null | \
  jq -r '.commit_decode'

echo
echo "✅ Verification complete!"
```

Run it:
```bash
chmod +x verify.sh
./verify.sh "hello"
```

## Troubleshooting

### "Cannot find UTXO"
- Check the address is correct
- Ensure the funding transaction is confirmed
- Verify you're on the correct network (testnet)

### "Transaction rejected"
- Wrong secret provided
- UTXO details incorrect
- Insufficient fee
- Asset ID mismatch

### "CMR doesn't match"
- Check you're using the exact same secret (case-sensitive!)
- Ensure SimplicityHL compiler version is the same
- Verify the contract source file is identical

---

**Pro Tip:** The safest way to verify is to create your own test puzzle with a known secret, then immediately solve it. This tests the entire flow end-to-end!
