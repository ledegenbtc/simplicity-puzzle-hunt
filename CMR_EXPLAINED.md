# CMR (Commitment Merkle Root) Explained

## Where is the CMR in the Output?

When you analyze a contract with `hal-simplicity`, the output looks like this:

```json
{
  "jets": "core",
  "commit_base64": "4udAoTaSzyTbpfsKMOJug7KsW54p4bFh5cH6dCXnMEM2KTi5gkBCBQgw...",
  "commit_decode": "(witness & iden); (((unit; const 0x2cf24dba...",
  "type_arrow": "1 → 1",

  "cmr": "88090352fc40bf24ec6b607b5b56bf7d2f9e47676223f2359fe9fda1cd448194",
  ^^^^^^
  HERE! This is the Commitment Merkle Root

  "liquid_address_unconf": "ex1prtk6va387h5kuvag7hwm2wjuzrwlutta4ghj489xglnrvvsr30hssx4qr6",
  "liquid_testnet_address_unconf": "tex1prtk6va387h5kuvag7hwm2wjuzrwlutta4ghj489xglnrvvsr30hs3l88j4",
  "is_redeem": false
}
```

### The CMR Field

**Field name:** `"cmr"`

**Value for "hello" puzzle:**
```
88090352fc40bf24ec6b607b5b56bf7d2f9e47676223f2359fe9fda1cd448194
```

This is a **32-byte (256-bit) hash** that uniquely identifies your compiled Simplicity program.

## How to Extract the CMR

### Method 1: Using jq
```bash
cargo run --bin export-program -- "hello" 2>/dev/null | \
  hal-simplicity simplicity simplicity info "$(cat)" 2>/dev/null | \
  jq -r '.cmr'
```

Output:
```
88090352fc40bf24ec6b607b5b56bf7d2f9e47676223f2359fe9fda1cd448194
```

### Method 2: Using grep
```bash
cargo run --bin export-program -- "hello" 2>/dev/null | \
  hal-simplicity simplicity simplicity info "$(cat)" 2>/dev/null | \
  grep '"cmr"'
```

Output:
```
  "cmr": "88090352fc40bf24ec6b607b5b56bf7d2f9e47676223f2359fe9fda1cd448194",
```

### Method 3: In Your Scripts
Our verification scripts already extract it:

```bash
CMR=$(cargo run --bin export-program -- "hello" 2>/dev/null | \
      hal-simplicity simplicity simplicity info "$(cat)" 2>/dev/null | \
      jq -r '.cmr')

echo "CMR: $CMR"
```

## Where is the CMR Used in Code?

### In `create_puzzle.rs`

```rust
// Line 82-83: Create the script from the CMR
let script = elements::Script::from(compiled.commit().cmr().as_ref().to_vec());
                                              ^^^^^^^^
                                              Getting the CMR!
```

The CMR is obtained by calling:
```rust
compiled.commit().cmr()
```

This returns the 32-byte CMR hash.

### In Taproot Construction

```rust
// The CMR becomes part of the Taproot script tree
let builder = elements::taproot::TaprootBuilder::new();
let builder = builder
    .add_leaf_with_ver(0, script, leaf_ver)  // <-- script contains the CMR
    .expect("tap tree should be valid");

let spend_info = builder
    .finalize(secp256k1::SECP256K1, internal_key)
    .expect("tap tree should be valid");

// Final address includes the CMR in its construction
let address = Address::p2tr(
    secp256k1::SECP256K1,
    spend_info.internal_key(),
    spend_info.merkle_root(),  // <-- Merkle root includes our CMR
    None,
    &AddressParams::LIQUID_TESTNET,
);
```

## CMR in the Address

The Taproot address is derived from:

```
┌─────────────────────────────────────────────┐
│         Taproot Address Construction        │
├─────────────────────────────────────────────┤
│                                             │
│  Internal Key (32 bytes)                    │
│    +                                        │
│  Merkle Root of Script Tree                 │
│    │                                        │
│    └─→ Contains: Leaf with CMR              │
│                                             │
│  = Taproot Output (tweaked key)             │
│                                             │
│  → tex1pd77euy... (Bech32m encoded)         │
│                                             │
└─────────────────────────────────────────────┘
```

The CMR is embedded in the script leaf, which is part of the Taproot tree.

## Visualizing the CMR Flow

```
┌──────────────────┐
│  Secret: "hello" │
└────────┬─────────┘
         │
         ▼
┌──────────────────────────────────────┐
│ SimplicityHL Compiler                │
│                                      │
│  1. Parse .simf file                 │
│  2. Compile with TARGET_HASH param   │
│  3. Generate Simplicity bytecode     │
│  4. Compute CMR (Merkle root)        │
└────────┬─────────────────────────────┘
         │
         ▼
┌──────────────────────────────────────────────────────┐
│  CMR: 88090352fc40bf24ec6b607b5b56bf7d2f9e479...    │  ◄── THIS VALUE
└────────┬─────────────────────────────────────────────┘
         │
         ├─────► Used in script leaf
         │
         ├─────► Part of Taproot tree
         │
         ├─────► Determines final address
         │
         └─────► Identifier for hal-simplicity
```

## CMR Properties

### 1. Deterministic
Same input → Same CMR
```bash
# Compile 3 times
./verify.sh "hello"

# All 3 CMRs will be identical:
# 88090352fc40bf24ec6b607b5b56bf7d2f9e47676223f2359fe9fda1cd448194
```

### 2. Unique per Secret
Different input → Different CMR
```bash
# "hello"
88090352fc40bf24ec6b607b5b56bf7d2f9e47676223f2359fe9fda1cd448194

# "satoshi"
944195448b3dd9ce03961bf8efb582ac3b8aeea0fe881d8c8de5ddf5fb461c07
                                                        ^^^^^^^^ Different!
```

### 3. Cryptographic Commitment
- Cannot be reversed (one-way hash)
- Any change to contract → Different CMR
- Binds funds to specific program logic

## All Output Fields Explained

```json
{
  "jets": "core",
  ^^^^^^^^^^^
  Which Simplicity jets are used (core = basic jets)

  "commit_base64": "4udAoTaSzyTbpfsKMOJug7KsW54p...",
  ^^^^^^^^^^^^^^^
  Base64-encoded Simplicity bytecode

  "commit_decode": "(witness & iden); (((unit; const 0x2cf24dba...",
  ^^^^^^^^^^^^^^^
  Human-readable program structure

  "type_arrow": "1 → 1",
  ^^^^^^^^^^^^
  Type signature: Unit → Unit

  "cmr": "88090352fc40bf24ec6b607b5b56bf7d2f9e47676223f2359fe9fda1cd448194",
  ^^^^^
  COMMITMENT MERKLE ROOT - Unique identifier for this contract

  "liquid_address_unconf": "ex1prtk6va387h5kuvag7hwm2wjuzrwlutta4ghj489xglnrvvsr30hssx4qr6",
  ^^^^^^^^^^^^^^^^^^^^^^^
  Address on Liquid mainnet (derived from CMR alone)

  "liquid_testnet_address_unconf": "tex1prtk6va387h5kuvag7hwm2wjuzrwlutta4ghj489xglnrvvsr30hs3l88j4",
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  Address on Liquid testnet (derived from CMR alone)

  "is_redeem": false
  ^^^^^^^^^^^
  false = commitment only (no witness data)
  true = includes witness data (satisfied program)
}
```

## CMR vs Address

### CMR-only Address (hal-simplicity)
```
CMR → Taproot construction → tex1prtk6va387h5kuvag7hwm2wjuzrwlutta4ghj489xglnrvvsr30hs3l88j4
```

### Actual Puzzle Address (create-puzzle)
```
Internal Key + CMR → Taproot construction → tex1pd77euywmg452m55mpfj0c5g434csl3ug8nl9y7k9gyc2fgh6xwfqdzyh7a
```

Different addresses, same CMR! Both are valid constructions.

## Practical Examples

### Check CMR for a puzzle
```bash
jq -r '.secret' puzzle_2cf24dba.json | xargs -I {} \
  sh -c 'cargo run --bin export-program -- "{}" 2>/dev/null | \
  hal-simplicity simplicity simplicity info "$(cat)" 2>/dev/null | \
  jq -r ".cmr"'
```

### Compare CMRs
```bash
echo "hello CMR:"
cargo run --bin export-program -- "hello" 2>/dev/null | \
  hal-simplicity simplicity simplicity info "$(cat)" 2>/dev/null | \
  jq -r '.cmr'

echo "satoshi CMR:"
cargo run --bin export-program -- "satoshi" 2>/dev/null | \
  hal-simplicity simplicity simplicity info "$(cat)" 2>/dev/null | \
  jq -r '.cmr'
```

### Verify CMR in Script
```bash
./check-puzzle.sh puzzle_2cf24dba.json | grep "CMR:"
```

Output:
```
✅ CMR: 88090352fc40bf24ec6b607b5b56bf7d2f9e47676223f2359fe9fda1cd448194
```

## Summary

**The CMR is:**
- ✅ In the `"cmr"` field of hal-simplicity output
- ✅ A 32-byte hex string
- ✅ Unique identifier for the compiled contract
- ✅ Embedded in the Taproot script tree
- ✅ Deterministic (same for identical programs)

**To find it:**
1. Compile: `cargo run --bin export-program -- "secret"`
2. Analyze: `hal-simplicity simplicity simplicity info "<base64>"`
3. Look for: `"cmr": "..."`

**Or use our scripts:**
- `./verify.sh "secret"` - Shows CMR
- `./check-puzzle.sh puzzle.json` - Shows CMR
- `./analyze_all.sh` - Shows all CMRs
