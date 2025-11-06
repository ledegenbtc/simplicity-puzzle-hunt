# Quick Reference - Contract Verification

## How to Match/Verify the Contract

### Method 1: Quick Verification (Recommended)
```bash
./check-puzzle.sh puzzle_2cf24dba.json
```

This will:
- ✅ Compile the contract from the secret
- ✅ Show the CMR (Commitment Merkle Root)
- ✅ Check if UTXOs exist on-chain
- ✅ Provide next steps to solve

### Method 2: Verify CMR Determinism
```bash
./verify.sh "hello"
```

This compiles the contract 3 times and verifies the CMR is identical each time.

### Method 3: Analyze Contract Structure
```bash
cargo run --bin export-program -- "hello" 2>/dev/null | \
  hal-simplicity simplicity simplicity info "$(cat)" | jq
```

Shows full contract details including:
- CMR
- Address (from CMR alone)
- Program structure
- Jets used

### Method 4: Analyze All Puzzles
```bash
./analyze_all.sh
```

Generates analysis for all puzzle_*.json files.

## Understanding the Results

### CMR (Commitment Merkle Root)
The unique identifier for the contract:
```
Secret "hello" → CMR: 88090352fc40bf24ec6b607b5b56bf7d2f9e47676223f2359fe9fda1cd448194
```

This CMR is:
- ✅ Deterministic (always the same for same secret)
- ✅ Unique per secret (different secret = different CMR)
- ✅ Embedded in the Taproot script tree

### Address Differences

**hal-simplicity address:**
```
tex1prtk6va387h5kuvag7hwm2wjuzrwlutta4ghj489xglnrvvsr30hs3l88j4
```

**Actual puzzle address:**
```
tex1pd77euywmg452m55mpfj0c5g434csl3ug8nl9y7k9gyc2fgh6xwfqdzyh7a
```

Why different?
- hal-simplicity uses CMR alone (unconfined address)
- Actual puzzle uses internal key + CMR in Taproot construction
- Both are valid! The CMR is what matters for verification

## Verification Checklist

To verify the contract matches:

- [ ] CMR is deterministic across compilations
- [ ] Program structure shows correct hash constant
- [ ] Contract uses expected jets (sha256, eq_256, verify)
- [ ] Type signature is `1 → 1`
- [ ] Can successfully solve puzzle (ultimate test!)

## Ultimate Verification: Solve It!

The best way to verify the contract works:

### 1. Check for UTXOs
```bash
./elements-cli.sh listunspent 0 9999999 '["tex1pd77euy..."]'
```

### 2. If UTXOs exist, get details
Look for:
- `txid` - Transaction ID
- `vout` - Output index
- `amount` - Amount in BTC
- `asset` - Asset ID

### 3. Edit solve_puzzle.rs
Update around line 120:
```rust
let txid_str = "YOUR_TXID";
let vout = 0;
let value_sats = 10000;
```

### 4. Get destination address
```bash
./elements-cli.sh getnewaddress
```

### 5. Solve!
```bash
cargo run --bin solve-puzzle -- puzzle_2cf24dba.json "hello" <your_address>
```

If it works → Contract verified! 🎉

## Common Verification Scenarios

### Scenario 1: Testing Your Own Puzzle

```bash
# Create
cargo run --bin create-puzzle -- "my_test_secret" 0.0001

# Verify
./check-puzzle.sh puzzle_<hash>.json

# Solve
# (update solve_puzzle.rs with UTXO info, then run)
cargo run --bin solve-puzzle -- puzzle_<hash>.json "my_test_secret" <addr>
```

### Scenario 2: Verifying Existing Puzzle

```bash
# Check status
./check-puzzle.sh puzzle_2cf24dba.json

# Analyze structure
cargo run --bin export-program -- "hello" 2>/dev/null | \
  hal-simplicity simplicity simplicity info "$(cat)"
```

### Scenario 3: Comparing Two Contracts

```bash
# Secret 1
CMR1=$(cargo run --bin export-program -- "hello" 2>/dev/null | \
       hal-simplicity simplicity simplicity info "$(cat)" | jq -r '.cmr')

# Secret 2
CMR2=$(cargo run --bin export-program -- "world" 2>/dev/null | \
       hal-simplicity simplicity simplicity info "$(cat)" | jq -r '.cmr')

echo "CMR for 'hello': $CMR1"
echo "CMR for 'world': $CMR2"

# Should be different!
```

## Files & Scripts

| File | Purpose |
|------|---------|
| `check-puzzle.sh` | Verify contract and check UTXO status |
| `verify.sh` | Test CMR determinism |
| `analyze_all.sh` | Analyze all puzzles |
| `src/bin/export_program.rs` | Export base64 contract |
| `VERIFY_CONTRACT.md` | Detailed verification guide |
| `SIMPLICITY_ANALYSIS.md` | Deep technical analysis |
| `CONTRACT_FLOW.md` | Visual execution flow |

## Troubleshooting

### "CMR doesn't match"
- Check secret is exactly the same (case-sensitive!)
- Ensure compiler version is identical
- Verify contract source code hasn't changed

### "No UTXOs found"
- Puzzle may be already solved
- Funding transaction may be pending
- Check if on correct network (testnet)

### "Transaction rejected"
- Wrong secret provided
- UTXO info incorrect
- Insufficient fees
- Asset ID mismatch

## Pro Tips

1. **Always verify CMR determinism first** - Run `./verify.sh <secret>`
2. **Test with your own puzzle** - Create, fund, and solve your own for full verification
3. **Compare multiple compilations** - CMR should never change for same input
4. **Check the program structure** - The hash constant should match your expected hash

## Quick Commands

```bash
# Verify a puzzle
./check-puzzle.sh puzzle_2cf24dba.json

# Test determinism
./verify.sh "hello"

# Analyze all
./analyze_all.sh

# Export and analyze
cargo run --bin export-program -- "hello" | \
  hal-simplicity simplicity simplicity info "$(cat)"

# Check UTXOs
./elements-cli.sh listunspent 0 9999999 '["<address>"]'

# Get new address
./elements-cli.sh getnewaddress
```

---

**Remember:** The ultimate verification is successfully solving the puzzle! If the transaction is accepted and you receive the funds, the contract is verified. ✅
