#!/bin/bash

echo "=== ANALYZING ALL PUZZLES ==="
echo

for puzzle in puzzle_*.json; do
    echo "=========================================="
    echo "File: $puzzle"
    echo "=========================================="

    # Extract details from puzzle file
    SECRET=$(jq -r '.secret' "$puzzle")
    HASH=$(jq -r '.hash' "$puzzle")
    ADDRESS=$(jq -r '.address' "$puzzle")
    AMOUNT=$(jq -r '.amount' "$puzzle")

    echo "Secret: $SECRET"
    echo "Hash: $HASH"
    echo "Address: $ADDRESS"
    echo "Amount: $AMOUNT L-BTC"
    echo

    # Generate and analyze the Simplicity program
    echo "Compiling and analyzing contract..."
    PROGRAM=$(cargo run --bin export-program -- "$SECRET" 2>/dev/null)
    ANALYSIS=$(echo "$PROGRAM" | hal-simplicity simplicity simplicity info "$PROGRAM" 2>/dev/null)

    echo "CMR: $(echo "$ANALYSIS" | jq -r '.cmr')"
    echo "Expected Address: $(echo "$ANALYSIS" | jq -r '.liquid_testnet_address_unconf')"
    echo
    echo "Program Structure:"
    echo "$ANALYSIS" | jq -r '.commit_decode' | head -c 200
    echo "..."
    echo
    echo
done

echo "Done!"
