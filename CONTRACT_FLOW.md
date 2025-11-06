# Simplicity Contract Execution Flow

## Visual Breakdown of Puzzle Contract

### High-Level Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                        PUZZLE CREATION                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Organizer chooses secret: "hello"                          │
│                                                                 │
│  2. Compute SHA256:                                            │
│     Input:  "hello"                                            │
│     Output: 0x2cf24dba5fb0a30e26e83b2ac5b9e29e...            │
│                                                                 │
│  3. Compile contract with TARGET_HASH = <hash>                 │
│                                                                 │
│  4. Create Taproot address from contract CMR                   │
│                                                                 │
│  5. Fund address with L-BTC                                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘

                              ↓

┌─────────────────────────────────────────────────────────────────┐
│                        FUNDS LOCKED                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Address: tex1pd77euywmg452m55mpfj0c5g434csl3ug8nl9y7k9...    │
│  Amount:  0.0001 L-BTC                                         │
│  Script:  Taproot with Simplicity contract leaf                │
│                                                                 │
│  Anyone can see:                                               │
│    ✓ The address                                               │
│    ✓ The amount                                                │
│    ✓ The hash target                                           │
│                                                                 │
│  Only organizer knows:                                         │
│    ✗ The secret preimage                                       │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘

                              ↓

┌─────────────────────────────────────────────────────────────────┐
│                     PUZZLE SOLVING ATTEMPT                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Solver guesses secret: "hello"                             │
│                                                                 │
│  2. Create transaction spending the UTXO                       │
│                                                                 │
│  3. Provide "hello" as witness data                            │
│                                                                 │
│  4. Broadcast to network                                       │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘

                              ↓

┌─────────────────────────────────────────────────────────────────┐
│                   ON-CHAIN VERIFICATION                         │
│                  (Executed by Bitcoin nodes)                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Stack starts: [witness_data]                                  │
│                                                                 │
│  Step 1: Extract SECRET from witness                           │
│    → Stack: [0x68656c6c6f]  ("hello" in hex)                  │
│                                                                 │
│  Step 2: Push TARGET_HASH constant                             │
│    → Stack: [SECRET, TARGET_HASH]                              │
│                                                                 │
│  Step 3: Initialize SHA256 context                             │
│    jet_sha_256_ctx_8_init                                      │
│    → Stack: [SECRET, TARGET_HASH, ctx]                         │
│                                                                 │
│  Step 4: Hash the SECRET                                       │
│    jet_sha_256_ctx_8_add_32(ctx, SECRET)                       │
│    → Stack: [TARGET_HASH, ctx']                                │
│                                                                 │
│  Step 5: Finalize hash                                         │
│    jet_sha_256_ctx_8_finalize(ctx')                            │
│    → Stack: [TARGET_HASH, computed_hash]                       │
│                                                                 │
│  Step 6: Compare hashes                                        │
│    jet_eq_256(computed_hash, TARGET_HASH)                      │
│    → Stack: [boolean_result]                                   │
│                                                                 │
│  Step 7: Verify result is TRUE                                 │
│    jet_verify                                                  │
│                                                                 │
│    If TRUE:  ✅ Transaction valid, funds released              │
│    If FALSE: ❌ Transaction rejected, funds stay locked        │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘

                              ↓

┌─────────────────────────────────────────────────────────────────┐
│                        OUTCOME                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Correct Secret ("hello"):                                     │
│    ✅ computed_hash == TARGET_HASH                             │
│    ✅ Transaction confirmed                                    │
│    ✅ Solver receives 0.0001 L-BTC                             │
│                                                                 │
│  Wrong Secret ("world"):                                       │
│    ❌ computed_hash != TARGET_HASH                             │
│    ❌ Transaction rejected                                     │
│    ❌ Funds remain locked                                      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Simplicity Combinator Breakdown

### Raw Decoded Program

```
(witness & iden);
(((unit; const 0x2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824) & iden);
(((IOH;
   (((unit; jet_sha_256_ctx_8_init) & iden);
   ((((OH & IH); jet_sha_256_ctx_8_add_32) & iden);
   (OH; jet_sha_256_ctx_8_finalize)))) & iden);
(((((OH & IOH); jet_eq_256); jet_verify) & unit); IH)))
```

### Annotated Version

```
Line 1: (witness & iden)
        │
        └─→ Extract witness data (the SECRET)

Line 2: (unit; const <HASH>)
        │
        └─→ Push TARGET_HASH onto stack

Line 3-5: SHA256 Computation
        │
        ├─→ jet_sha_256_ctx_8_init    (create hasher)
        ├─→ jet_sha_256_ctx_8_add_32  (add SECRET)
        └─→ jet_sha_256_ctx_8_finalize (get digest)

Line 6: jet_eq_256
        │
        └─→ Compare computed_hash with TARGET_HASH
            Returns: true or false

Line 7: jet_verify
        │
        └─→ Assert the comparison was true
            If false: abort transaction
            If true:  continue execution (success)
```

## Comparison: Different Secrets

### Secret: "hello"

```
Input:        "hello"
SHA256:       2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
CMR:          88090352fc40bf24ec6b607b5b56bf7d2f9e47676223f2359fe9fda1cd448194
Address:      tex1pd77euywmg452m55mpfj0c5g434csl3ug8nl9y7k9gyc2fgh6xwfqdzyh7a

Contract:     (witness & iden); ((unit; const 0x2cf24dba...) & iden); ...
              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
              Only this hash constant differs between contracts
```

### Secret: "satoshi"

```
Input:        "satoshi"
SHA256:       da2876b3eb31edb4436fa4650673fc6f01f90de2f1793c4ec332b2387b09726f
CMR:          944195448b3dd9ce03961bf8efb582ac3b8aeea0fe881d8c8de5ddf5fb461c07
Address:      tex1pwq857v90v7y9gup4nkkcuqa03rw5j4q0mzflpzj7a5nzjllhq9fsnex22e

Contract:     (witness & iden); ((unit; const 0xda2876b3...) & iden); ...
                                                ^^^^^^^^^^^
                                        Different hash = Different CMR
```

## Security Properties Visualized

```
┌─────────────────────┐
│   PUBLIC CHAIN      │
│  (Everyone can see) │
└─────────────────────┘
        │
        ├─→ Address:     tex1pd77euy...
        ├─→ Amount:      0.0001 L-BTC
        ├─→ Target Hash: 0x2cf24dba...
        ├─→ Contract:    [Simplicity bytecode]
        │
        └─→ Can verify the rules, but...
            Cannot derive the secret from hash!
            (SHA256 preimage resistance)

┌─────────────────────┐
│   PRIVATE           │
│ (Organizer's secret)│
└─────────────────────┘
        │
        └─→ Secret:     "hello"

            This is the ONLY information needed
            to claim the funds!
```

## Why This Is Trustless

```
Traditional Puzzle:
┌─────────┐           ┌──────────┐          ┌────────┐
│ Solver  │ → guess → │ Organizer│ → check →│ Result │
└─────────┘           └──────────┘          └────────┘
                           ↑
                      Must trust!

Simplicity Puzzle:
┌─────────┐           ┌──────────┐          ┌────────┐
│ Solver  │ → guess → │ Bitcoin  │ → check →│ Result │
└─────────┘           │ Network  │          └────────┘
                      └──────────┘
                           ↑
                    Math guarantees!
                    (no trust needed)
```

## Gas/Cost Analysis

Simplicity jets are optimized opcodes:
- `jet_sha_256_ctx_8_init`: ~5 weight units
- `jet_sha_256_ctx_8_add_32`: ~10 weight units
- `jet_sha_256_ctx_8_finalize`: ~15 weight units
- `jet_eq_256`: ~5 weight units
- `jet_verify`: ~2 weight units

**Total: ~37 weight units**

Compare to implementing SHA256 from scratch:
- Would require ~1000+ combinators
- Much higher weight and cost

Jets make complex crypto operations practical!

## Common Questions

### Q: Why is the address different from hal-simplicity output?

**A:** The puzzle uses a specific internal key in the Taproot construction. The `hal-simplicity` address is derived from CMR only (unconfined). Both are valid - just different construction methods.

### Q: Can someone see my secret if I solve the puzzle?

**A:** Yes! Once you broadcast the transaction, the secret becomes public (in the witness data). This is a race - whoever broadcasts first wins.

### Q: Can the organizer change the rules after funding?

**A:** No! The rules are committed in the CMR which determines the address. Changing rules = different address = different funds.

### Q: What if two people find the secret at the same time?

**A:** First transaction to be confirmed wins. Use Replace-By-Fee (RBF) or high fees for priority.

---

*Generated by analyzing compiled Simplicity contracts with hal-simplicity*
