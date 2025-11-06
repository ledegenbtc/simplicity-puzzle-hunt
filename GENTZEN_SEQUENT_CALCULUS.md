# Gentzen's Sequent Calculus for Simplicity Contracts

This document presents the Gentzen's sequent calculus derivations for the Simplicity programs implemented in this project.

## Notation

In Gentzen's sequent calculus for Simplicity, we use the following notation:

```
Γ ⊢ t : A ⊢ B
```

Where:
- `Γ` is the typing context (environment)
- `t` is the Simplicity term
- `A` is the input type
- `B` is the output type
- `⊢` indicates the sequent turnstile

## Simplicity Types

The basic types used in our contracts:
- `1` (unit type)
- `2` (sum of two units, boolean-like)
- `u256` (256-bit unsigned integer)
- `Ctx8` (SHA-256 context type)
- `A × B` (product type)
- `A + B` (sum type)

## Contract: puzzle_jackpot.simf

### Overview

The puzzle jackpot contract implements a simple hash-lock mechanism where:
1. A secret preimage (u256) is provided as witness
2. A target hash (u256) is provided as parameter
3. The program computes SHA256(secret)
4. The program asserts that computed_hash == target_hash

### Full Program Derivation Tree

```
                                                    [WITNESS]
                                            ——————————————————————————
                                            Γ ⊢ SECRET : 1 ⊢ u256
                                                    [PARAM]
                                            ——————————————————————————
                                            Γ ⊢ TARGET_HASH : 1 ⊢ u256


                    [sha2 function - expanded below]
        ————————————————————————————————————————————————————————————
        Γ ⊢ sha2(SECRET) : 1 ⊢ u256


                                [IDEN]
                        ————————————————————————
                        Γ ⊢ iden : u256 ⊢ u256


        [COMP]                                              [PAIR]
Γ ⊢ SECRET : 1 ⊢ u256    Γ ⊢ TARGET_HASH : 1 ⊢ u256    Γ ⊢ (sha2(SECRET), TARGET_HASH) : 1 ⊢ u256 × u256


                                        [JET: eq_256]
                        ————————————————————————————————————————————————
                        Γ ⊢ jet::eq_256 : u256 × u256 ⊢ 2


                                        [COMP]
        ————————————————————————————————————————————————————————————————
        Γ ⊢ jet::eq_256(sha2(SECRET), TARGET_HASH) : 1 ⊢ 2


                                        [ASSERT]
        ————————————————————————————————————————————————————————————————
        Γ ⊢ assert!(jet::eq_256(sha2(SECRET), TARGET_HASH)) : 1 ⊢ 1
```

### Component: sha2 Function

The `sha2` function computes SHA-256 of a u256 value:

```
                                    [JET: sha_256_ctx_8_init]
                            ——————————————————————————————————————————
                            Γ ⊢ jet::sha_256_ctx_8_init : 1 ⊢ Ctx8


                    [IDEN]                              [PAIR]
            ————————————————————            ——————————————————————————
            Γ ⊢ iden : u256 ⊢ u256         Γ ⊢ (Ctx8, u256) : A ⊢ Ctx8 × u256


                                [JET: sha_256_ctx_8_add_32]
                        ————————————————————————————————————————————————
                        Γ ⊢ jet::sha_256_ctx_8_add_32 : Ctx8 × u256 ⊢ Ctx8


                                    [COMP]
        ————————————————————————————————————————————————————————————————————
        Γ ⊢ jet::sha_256_ctx_8_add_32(init(), preimage) : u256 ⊢ Ctx8


                                [JET: sha_256_ctx_8_finalize]
                        ————————————————————————————————————————————————
                        Γ ⊢ jet::sha_256_ctx_8_finalize : Ctx8 ⊢ u256


                                        [COMP]
        ————————————————————————————————————————————————————————————————————
        Γ ⊢ sha2 : u256 ⊢ u256
```

### Detailed Inference Rules

#### 1. Identity (IDEN)
```
————————————————————————
Γ ⊢ iden : A ⊢ A
```

The identity combinator passes its input through unchanged.

#### 2. Composition (COMP)
```
Γ ⊢ s : A ⊢ B    Γ ⊢ t : B ⊢ C
————————————————————————————————
Γ ⊢ comp s t : A ⊢ C
```

Composition chains two Simplicity expressions, where the output of `s` becomes the input to `t`.

#### 3. Unit (UNIT)
```
————————————————————————
Γ ⊢ unit : A ⊢ 1
```

The unit combinator produces the unit value, discarding its input.

#### 4. Injections (INJL, INJR)
```
Γ ⊢ t : A ⊢ B                    Γ ⊢ t : A ⊢ C
————————————————————————          ————————————————————————
Γ ⊢ injl t : A ⊢ B + C           Γ ⊢ injr t : A ⊢ B + C
```

Injections construct sum types (either left or right variant).

#### 5. Pair (PAIR)
```
Γ ⊢ s : A ⊢ B    Γ ⊢ t : A ⊢ C
————————————————————————————————
Γ ⊢ pair s t : A ⊢ B × C
```

Pair constructs product types from two expressions with the same input.

#### 6. Take (TAKE)
```
Γ ⊢ t : A ⊢ C
————————————————————————
Γ ⊢ take t : A × B ⊢ C
```

Take projects the first component of a product type and applies `t`.

#### 7. Drop (DROP)
```
Γ ⊢ t : B ⊢ C
————————————————————————
Γ ⊢ drop t : A × B ⊢ C
```

Drop projects the second component of a product type and applies `t`.

#### 8. Case (CASE)
```
Γ ⊢ s : A × B ⊢ D    Γ ⊢ t : A × C ⊢ D
————————————————————————————————————————
Γ ⊢ case s t : A × (B + C) ⊢ D
```

Case performs pattern matching on sum types.

#### 9. Witness (WITNESS)
```
w : B ∈ W
————————————————————————
Γ, W ⊢ witness w : A ⊢ B
```

Witness reads a value from the witness data. In our contract:
```
————————————————————————————————
Γ ⊢ witness::SECRET : 1 ⊢ u256
```

#### 10. Jet (JET)
```
jet_name : A → B
————————————————————————————————
Γ ⊢ jet::jet_name : A ⊢ B
```

Jets are primitive operations. In our contract:

```
————————————————————————————————————————
Γ ⊢ jet::sha_256_ctx_8_init : 1 ⊢ Ctx8

————————————————————————————————————————————————————
Γ ⊢ jet::sha_256_ctx_8_add_32 : Ctx8 × u256 ⊢ Ctx8

————————————————————————————————————————————————
Γ ⊢ jet::sha_256_ctx_8_finalize : Ctx8 ⊢ u256

————————————————————————————————————————
Γ ⊢ jet::eq_256 : u256 × u256 ⊢ 2
```

#### 11. Assertion (ASSERT)
```
Γ ⊢ t : A ⊢ 2
————————————————————————
Γ ⊢ assert t : A ⊢ 1
```

Assertion checks that the result is `2::1` (true). If the result is `2::0` (false), execution fails.

In our contract:
```
Γ ⊢ jet::eq_256(sha2(SECRET), TARGET_HASH) : 1 ⊢ 2
——————————————————————————————————————————————————————
Γ ⊢ assert!(jet::eq_256(sha2(SECRET), TARGET_HASH)) : 1 ⊢ 1
```

### Complete Derivation with All Steps

Here's the complete derivation of the puzzle_jackpot program showing all intermediate steps:

```
[Step 1: Read witness SECRET]
————————————————————————————————
Γ ⊢ SECRET : 1 ⊢ u256


[Step 2: Initialize SHA-256 context]
————————————————————————————————————————
Γ ⊢ jet::sha_256_ctx_8_init : 1 ⊢ Ctx8


[Step 3: Add preimage to context]
Γ ⊢ init : 1 ⊢ Ctx8    Γ ⊢ SECRET : 1 ⊢ u256
————————————————————————————————————————————— [PAIR]
Γ ⊢ (init, SECRET) : 1 ⊢ Ctx8 × u256

Γ ⊢ (init, SECRET) : 1 ⊢ Ctx8 × u256    Γ ⊢ jet::sha_256_ctx_8_add_32 : Ctx8 × u256 ⊢ Ctx8
——————————————————————————————————————————————————————————————————————————————————————————— [COMP]
Γ ⊢ jet::sha_256_ctx_8_add_32(init, SECRET) : 1 ⊢ Ctx8


[Step 4: Finalize hash]
Γ ⊢ add_32(init, SECRET) : 1 ⊢ Ctx8    Γ ⊢ jet::sha_256_ctx_8_finalize : Ctx8 ⊢ u256
——————————————————————————————————————————————————————————————————————————————————————— [COMP]
Γ ⊢ sha2(SECRET) : 1 ⊢ u256


[Step 5: Read parameter TARGET_HASH]
————————————————————————————————————
Γ ⊢ TARGET_HASH : 1 ⊢ u256


[Step 6: Create pair for comparison]
Γ ⊢ sha2(SECRET) : 1 ⊢ u256    Γ ⊢ TARGET_HASH : 1 ⊢ u256
—————————————————————————————————————————————————————————— [PAIR]
Γ ⊢ (sha2(SECRET), TARGET_HASH) : 1 ⊢ u256 × u256


[Step 7: Compare hashes]
Γ ⊢ (sha2(SECRET), TARGET_HASH) : 1 ⊢ u256 × u256    Γ ⊢ jet::eq_256 : u256 × u256 ⊢ 2
——————————————————————————————————————————————————————————————————————————————————————— [COMP]
Γ ⊢ jet::eq_256(sha2(SECRET), TARGET_HASH) : 1 ⊢ 2


[Step 8: Assert equality (final step)]
Γ ⊢ jet::eq_256(sha2(SECRET), TARGET_HASH) : 1 ⊢ 2
——————————————————————————————————————————————————————— [ASSERT]
Γ ⊢ assert!(jet::eq_256(sha2(SECRET), TARGET_HASH)) : 1 ⊢ 1
```

## Type Safety and Correctness

The Gentzen sequent calculus ensures that:

1. **Type Safety**: Every expression has a well-defined input and output type
2. **Composability**: Expressions can be composed only if types match
3. **Correctness**: The derivation proves that the program is well-typed
4. **Verification**: Each step follows from valid inference rules

For the puzzle_jackpot contract:
- Input type: `1` (unit, as the contract doesn't require transaction data)
- Output type: `1` (unit, after successful assertion)
- The program succeeds if and only if SHA256(SECRET) == TARGET_HASH

## Semantic Interpretation

The Gentzen sequent calculus not only proves type correctness but also provides semantic meaning:

```
⟦Γ ⊢ t : A ⊢ B⟧ : ⟦A⟧ → ⟦B⟧
```

For our puzzle contract:
```
⟦Γ ⊢ puzzle : 1 ⊢ 1⟧ : Unit → Unit
```

The function succeeds (returns Unit) if the witness provides the correct preimage, and fails otherwise.

## Bitcoin Integration

When integrated into a Bitcoin/Elements transaction:
1. The Simplicity program is committed to via Taproot
2. The witness data (SECRET) is provided in the transaction witness
3. The program executes during transaction validation
4. Success allows the transaction to spend the UTXO
5. Failure causes transaction rejection

## References

- **Simplicity Language Specification**: [https://github.com/BlockstreamResearch/simplicity](https://github.com/BlockstreamResearch/simplicity)
- **Gentzen's Sequent Calculus**: Original work by Gerhard Gentzen (1934-1935)
- **Type Theory and Formal Proof**: Nederpelt & Geuvers
- **Blockstream Research**: Simplicity technical papers

## Implementation Files

The contracts analyzed in this document are implemented in:
- Contract source: `SimplicityHL/examples/puzzle_jackpot.simf`
- Creation tool: `src/bin/create_puzzle.rs`
- Solving tool: `src/bin/solve_puzzle.rs`
- Export tool: `src/bin/export_program.rs`
