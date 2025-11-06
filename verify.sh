#!/bin/bash

SECRET="$1"

if [ -z "$SECRET" ]; then
    echo "Usage: $0 <secret>"
    echo
    echo "Example:"
    echo "  $0 \"hello\""
    exit 1
fi

echo "=== VERIFYING CONTRACT FOR SECRET: $SECRET ==="
echo

# Compile 3 times
echo "Compiling 3 times to verify determinism..."
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

# Get full analysis
echo
echo "=== FULL CONTRACT ANALYSIS ==="
ANALYSIS=$(cargo run --bin export-program -- "$SECRET" 2>/dev/null | \
           hal-simplicity simplicity simplicity info "$(cat)" 2>/dev/null)

echo "CMR:     $(echo "$ANALYSIS" | jq -r '.cmr')"
echo "Address: $(echo "$ANALYSIS" | jq -r '.liquid_testnet_address_unconf')"
echo "Type:    $(echo "$ANALYSIS" | jq -r '.type_arrow')"
echo "Jets:    $(echo "$ANALYSIS" | jq -r '.jets')"
echo

# Show program structure
echo "=== CONTRACT STRUCTURE ==="
echo "$ANALYSIS" | jq -r '.commit_decode'
echo

# Show full JSON
echo "=== FULL JSON OUTPUT ==="
echo "$ANALYSIS" | jq '.'
echo

echo "✅ Verification complete!"
