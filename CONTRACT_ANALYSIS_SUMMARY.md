# Contract Analysis Summary - Puzzle Hunt

## Analysis Results

Using `hal-simplicity` to analyze the compiled Simplicity contracts created for the puzzle hunt.

### Puzzle 1: "hello"

**Puzzle Details:**
- Secret: `hello`
- Hash: `0x2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824`
- Address: `tex1pd77euywmg452m55mpfj0c5g434csl3ug8nl9y7k9gyc2fgh6xwfqdzyh7a`
- Amount: 0.0001 L-BTC

**Contract Analysis:**
- CMR: `88090352fc40bf24ec6b607b5b56bf7d2f9e47676223f2359fe9fda1cd448194`
- Base Address (from CMR): `tex1prtk6va387h5kuvag7hwm2wjuzrwlutta4ghj489xglnrvvsr30hs3l88j4`

### Puzzle 2: "satoshi"

**Puzzle Details:**
- Secret: `satoshi`
- Hash: `0xda2876b3eb31edb4436fa4650673fc6f01f90de2f1793c4ec332b2387b09726f`
- Address: `tex1pwq857v90v7y9gup4nkkcuqa03rw5j4q0mzflpzj7a5nzjllhq9fsnex22e`
- Amount: 0.0001 L-BTC

**Contract Analysis:**
- CMR: `944195448b3dd9ce03961bf8efb582ac3b8aeea0fe881d8c8de5ddf5fb461c07`
- Base Address (from CMR): `tex1pclczcdfmjnklsdl4037knzln39zgw80xxp42ku5l6ksdu6h0q3lq5tda9v`

### Puzzle 3: "meusecreto"

**Puzzle Details:**
- Secret: `meusecreto`
- Hash: `0xffd173ce0eda7ec7023ebbab6a7eced6df6cd79b389b899ed951fc60e3e6fcdf`
- Address: `tex1ptn74qweammpw4szzqgq2jfhnp3aghzk39nfdaew4uq8xz2efnwtswxmtwf`
- Amount: 0.0001 L-BTC

**Contract Analysis:**
- CMR: `b578da48fb2a1afaf6212511df6fd5219c4cd6ed3e3a64e907b83aaa20731b8c`
- Base Address (from CMR): `tex1pzd8v747c9n2nxjtryf94u36d8yhu5v3m8q2krffly8s7gpvycyfs8zc6f7`

## Key Observations

### 1. Address Differences

The actual puzzle addresses differ from the addresses derived directly from the CMR because:
- The actual addresses use a specific **internal key** (taproot key path)
- The `hal-simplicity` addresses are derived from CMR alone (unconfined addresses)
- Both approaches are valid - they represent different taproot address constructions

### 2. Unique CMRs

Each puzzle has a unique CMR because:
- The `TARGET_HASH` is compiled as a constant into the contract
- Different hash → Different bytecode → Different CMR
- This ensures each puzzle is a truly distinct smart contract

### 3. Program Structure (All Puzzles)

All three puzzles share the same logical structure:

```
(witness & iden);                          # Take witness data (SECRET)
(const <TARGET_HASH> & iden);              # Push the target hash
(                                          # SHA256 computation:
  jet_sha_256_ctx_8_init                   #   Initialize
  jet_sha_256_ctx_8_add_32                 #   Add SECRET
  jet_sha_256_ctx_8_finalize               #   Finalize
);
(jet_eq_256; jet_verify);                  # Compare & verify
```

The only difference between contracts is the `<TARGET_HASH>` constant value.

### 4. Jets Used

All contracts use these Simplicity jets:
- `jet_sha_256_ctx_8_init` - Initialize SHA256 hasher
- `jet_sha_256_ctx_8_add_32` - Hash 32 bytes of input
- `jet_sha_256_ctx_8_finalize` - Complete hashing
- `jet_eq_256` - Compare two 256-bit values
- `jet_verify` - Assert condition is true (fails transaction if false)

### 5. Type Signature

All contracts have the same type: `1 → 1`
- Input: Unit type (the transaction environment provides all needed data)
- Output: Unit type (success is indicated by not failing)

## Security Properties

### Cryptographic Guarantees

1. **Preimage Resistance**: Cannot derive secret from hash (SHA256 security)
2. **Deterministic**: Same secret always produces same result
3. **Verifiable**: Anyone can verify the contract logic on-chain
4. **Atomic**: All-or-nothing execution (no partial states)

### Attack Surface

The only way to claim funds:
1. Find the preimage (secret) that hashes to TARGET_HASH
2. Provide it as witness data when spending
3. The network validates the hash match

No trusted parties, no off-chain coordination needed.

## Tools Used

### 1. SimplicityHL Compiler
- Compiles high-level `.simf` programs to Simplicity bytecode
- Located in parent directory: `/Users/felipe/Desktop/hub/blockchain/SimplicityHL`

### 2. hal-simplicity
- Command-line tool for analyzing Simplicity programs
- Installed via: `cargo install hal-simplicity`
- Usage: `hal-simplicity simplicity simplicity info <base64-program>`

### 3. Export Program Utility
- Custom binary created: `src/bin/export_program.rs`
- Compiles puzzle contracts and outputs base64-encoded programs
- Usage: `cargo run --bin export-program -- <secret>`

## Verification Steps

To verify these results yourself:

1. **Install hal-simplicity:**
   ```bash
   cargo install hal-simplicity
   ```

2. **Export a puzzle program:**
   ```bash
   cargo run --bin export-program -- "hello" > puzzle.b64
   ```

3. **Analyze with hal-simplicity:**
   ```bash
   hal-simplicity simplicity simplicity info "$(cat puzzle.b64)"
   ```

4. **Compare CMRs:**
   - The CMR should match for identical secrets
   - Different secrets produce different CMRs

## Educational Value

This analysis demonstrates:

1. **Transparent Smart Contracts**: The entire program logic is visible and analyzable
2. **Deterministic Compilation**: Same source + parameters = same bytecode
3. **Cryptographic Commitments**: CMR binds funds to specific contract logic
4. **On-Chain Verification**: Bitcoin network validates all execution

## Next Steps

- **For Developers**: Modify `puzzle_jackpot.simf` to create custom puzzle logic
- **For Players**: Use this analysis to understand how the puzzles work
- **For Organizers**: See `SIMPLICITY_ANALYSIS.md` for security best practices

## Files Generated

- `SIMPLICITY_ANALYSIS.md` - Detailed technical analysis
- `CONTRACT_ANALYSIS_SUMMARY.md` - This file
- `analyze_all.sh` - Script to analyze all puzzles
- `src/bin/export_program.rs` - Tool to export compiled programs

---

*Analysis performed on: 2025-11-05*
*Tools: hal-simplicity v0.1.0, SimplicityHL compiler*
