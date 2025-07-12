#!/bin/bash
set -e

echo "=== Direct E2E Test ==="
echo "Testing without nix wrapper"
echo ""

# Check if tools are available
if ! command -v solana >/dev/null 2>&1; then
    echo "Error: solana CLI not found"
    echo "Please run this test through nix: nix run ./tests/e2e"
    exit 1
fi

# Run the actual test
bash run_e2e_test.sh