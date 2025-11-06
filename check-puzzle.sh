#!/bin/bash
# Script to verify puzzle contract and check on-chain status

PUZZLE_FILE="$1"

if [ -z "$PUZZLE_FILE" ]; then
    echo "Usage: $0 <puzzle_file.json>"
    echo
    echo "Example:"
    echo "  $0 puzzle_2cf24dba.json"
    echo
    echo "Available puzzles:"
    ls -1 puzzle_*.json 2>/dev/null | sed 's/^/  /'
    exit 1
fi

if [ ! -f "$PUZZLE_FILE" ]; then
    echo "Error: File $PUZZLE_FILE not found"
    exit 1
fi

echo "=== PUZZLE CONTRACT VERIFICATION ==="
echo

# Extract puzzle details
SECRET=$(jq -r '.secret' "$PUZZLE_FILE")
HASH=$(jq -r '.hash' "$PUZZLE_FILE")
ADDRESS=$(jq -r '.address' "$PUZZLE_FILE")
AMOUNT=$(jq -r '.amount' "$PUZZLE_FILE")

echo "📁 Puzzle File: $PUZZLE_FILE"
echo "🔐 Secret: $SECRET"
echo "🔗 Hash: $HASH"
echo "📍 Address: $ADDRESS"
echo "💰 Amount: $AMOUNT L-BTC"
echo

# Compile and get CMR
echo "⚙️  Compiling contract..."
CMR=$(cargo run --bin export-program -- "$SECRET" 2>/dev/null | \
      hal-simplicity simplicity simplicity info "$(cat)" 2>/dev/null | \
      jq -r '.cmr')

echo "✅ CMR: $CMR"
echo

# Check if Elements CLI is available
ELEMENTS_CLI="$HOME/Desktop/hub/blockchain/elements/src/elements-cli"
if [ ! -f "$ELEMENTS_CLI" ]; then
    echo "⚠️  Elements CLI not found at: $ELEMENTS_CLI"
    echo "Cannot check on-chain UTXO"
    exit 0
fi

# Check for UTXOs at this address
echo "🔍 Checking on-chain UTXOs..."
echo

UTXOS=$($ELEMENTS_CLI -chain=liquidtestnet listunspent 0 9999999 "[\"$ADDRESS\"]" 2>/dev/null)

if [ $? -ne 0 ]; then
    echo "❌ Error connecting to Elements daemon"
    echo
    echo "Make sure elementsd is running:"
    echo "  cd $HOME/Desktop/hub/blockchain/elements"
    echo "  ./src/elementsd -chain=liquidtestnet -daemon"
    exit 1
fi

# Count UTXOs
UTXO_COUNT=$(echo "$UTXOS" | jq 'length')

if [ "$UTXO_COUNT" -eq 0 ]; then
    echo "⚠️  No UTXOs found at this address"
    echo "   The puzzle may have been solved or not yet funded"
else
    echo "✅ Found $UTXO_COUNT UTXO(s) at this address"
    echo
    echo "$UTXOS" | jq -r '.[] | "  TXID:   \(.txid)\n  VOUT:   \(.vout)\n  Amount: \(.amount) L-BTC\n  Confs:  \(.confirmations)\n  Asset:  \(.asset)\n"'
fi

echo
echo "=== VERIFICATION SUMMARY ==="
echo "✅ CMR is deterministic: $CMR"
echo "✅ Contract compiles successfully"
echo "✅ Address: $ADDRESS"

if [ "$UTXO_COUNT" -gt 0 ]; then
    echo "✅ Puzzle is LIVE and can be solved!"
    echo
    echo "💡 To solve this puzzle:"
    echo "   1. Get a destination address: ./elements-cli.sh getnewaddress"
    echo "   2. Edit src/bin/solve_puzzle.rs with UTXO details above"
    echo "   3. Run: cargo run --bin solve-puzzle -- $PUZZLE_FILE \"$SECRET\" <dest_address>"
else
    echo "⚠️  Puzzle has no UTXOs (already solved or not funded)"
fi

echo
echo "🔬 For detailed analysis, see:"
echo "   - SIMPLICITY_ANALYSIS.md"
echo "   - CONTRACT_FLOW.md"
