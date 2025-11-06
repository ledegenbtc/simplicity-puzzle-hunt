# Simplicity Contract Analysis - Puzzle Jackpot

## Overview

This document contains the analysis of the compiled Simplicity smart contract used in the Bitcoin Puzzle Hunt game.

## Contract Source (`examples/puzzle_jackpot.simf`)

The contract implements a hash-lock puzzle where:
1. Takes a `SECRET` as witness data (provided by the solver)
2. Has a `TARGET_HASH` as a compile-time parameter (set when creating the puzzle)
3. Computes `sha256(SECRET)` using Simplicity jets
4. Verifies that `computed_hash == TARGET_HASH`
5. If verification passes, the UTXO can be spent

## Analysis for Secret "hello"

**Input:**
- Secret: `hello`
- Hash (SHA256): `0x2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824`

**hal-simplicity Output:**

```json
{
  "jets": "core",
  "commit_base64": "4udAoTaSzyTbpfsKMOJug7KsW54p4bFh5cH6dCXnMEM2KTi5gkBCBQgw5OrIECZIMPpFGq5AhAoNo1YggGAG0DgRAoNwc3CDhVm3QCMAhIoPxQCAbQOJQOKw",
  "commit_decode": "(witness  & iden); (((unit; const 0x2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824 ) & iden); (((IOH; (((unit; jet_sha_256_ctx_8_init ) & iden); ((((OH & IH); jet_sha_256_ctx_8_add_32 ) & iden); (OH; jet_sha_256_ctx_8_finalize )))) & iden); (((((OH & IOH); jet_eq_256 ); jet_verify ) & unit); IH)))",
  "type_arrow": "1 → 1",
  "cmr": "88090352fc40bf24ec6b607b5b56bf7d2f9e47676223f2359fe9fda1cd448194",
  "liquid_address_unconf": "ex1prtk6va387h5kuvag7hwm2wjuzrwlutta4ghj489xglnrvvsr30hssx4qr6",
  "liquid_testnet_address_unconf": "tex1prtk6va387h5kuvag7hwm2wjuzrwlutta4ghj489xglnrvvsr30hs3l88j4",
  "is_redeem": false
}
```

## Decoded Program Structure

The `commit_decode` field shows the actual Simplicity combinators:

```
(witness & iden);
(((unit; const 0x2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824) & iden);
(((IOH;
   (((unit; jet_sha_256_ctx_8_init) & iden);
   ((((OH & IH); jet_sha_256_ctx_8_add_32) & iden);
   (OH; jet_sha_256_ctx_8_finalize)))) & iden);
(((((OH & IOH); jet_eq_256); jet_verify) & unit); IH)))
```

### Breakdown:

1. **`witness & iden`** - Takes witness data (the SECRET) from the stack
2. **`const 0x2cf24dba...`** - Pushes the TARGET_HASH constant (hardcoded at compile time)
3. **SHA256 Jets:**
   - `jet_sha_256_ctx_8_init` - Initialize SHA256 context
   - `jet_sha_256_ctx_8_add_32` - Add 32 bytes (the SECRET) to the hash
   - `jet_sha_256_ctx_8_finalize` - Finalize and produce the hash digest
4. **Verification:**
   - `jet_eq_256` - Compare computed hash with TARGET_HASH
   - `jet_verify` - Assert equality (transaction fails if false)

## Key Properties

### Commitment Merkle Root (CMR)
```
88090352fc40bf24ec6b607b5b56bf7d2f9e47676223f2359fe9fda1cd448194
```

The CMR is the cryptographic commitment to this specific program. It's embedded in the Taproot script tree and determines the final address.

### Type Arrow
```
1 → 1
```

The program takes unit type as input and produces unit type as output (side-effect based verification).

### Jets Used
- `core` - Uses core Simplicity jets (SHA256, equality, verification)

### Addresses

**Liquid Mainnet (unconfined):**
```
ex1prtk6va387h5kuvag7hwm2wjuzrwlutta4ghj489xglnrvvsr30hssx4qr6
```

**Liquid Testnet (unconfined):**
```
tex1prtk6va387h5kuvag7hwm2wjuzrwlutta4ghj489xglnrvvsr30hs3l88j4
```

Note: These addresses are derived directly from the CMR. The actual puzzle addresses will differ because they include a different internal key.

## Security Analysis

### What Makes This Secure?

1. **Preimage Resistance**: SHA256 is cryptographically secure - you cannot derive the secret from the hash
2. **On-Chain Verification**: The Bitcoin/Liquid network verifies the hash match - no trust required
3. **Atomic**: Either the correct secret is provided and funds are claimed, or the transaction is rejected
4. **Transparent**: Anyone can verify the contract logic and rules

### Attack Vectors?

- **Brute Force**: Only viable if the secret is weak (short, predictable)
- **Rainbow Tables**: Pre-computed hash tables for common passwords
- **Social Engineering**: Tricking the organizer into revealing the secret

### Best Practices for Organizers:

1. Use long, random secrets (not dictionary words)
2. Never reuse secrets across puzzles
3. Keep the secret truly secret until the game ends
4. Consider using secrets with high entropy (random hex strings)

## Comparison: Different Secrets

The program structure remains identical for different secrets - only the `TARGET_HASH` constant changes:

- Secret "hello" → Hash `0x2cf24dba...` → CMR `88090352...`
- Secret "satoshi" → Hash `0xa0dc65ff...` → CMR (different)
- Secret "bitcoin" → Hash (different) → CMR (different)

Each unique TARGET_HASH produces a different CMR, and thus a different Taproot address.

## Tools Used

- **SimplicityHL Compiler**: Compiles high-level Simplicity code to bytecode
- **hal-simplicity**: Analyzes and decodes Simplicity programs
- **Elements**: Creates Taproot addresses from the compiled program

## References

- [Simplicity Language Specification](https://github.com/BlockstreamResearch/simplicity)
- [Elements Documentation](https://elementsproject.org/)
- [SimplicityHL Compiler](https://github.com/BlockstreamResearch/rust-simplicity)
