#!/bin/bash
set -e

echo "=== E2E Test Setup Verification ==="
echo ""

# Check if required files exist
echo "Checking files..."
for file in "flake.nix" "run_e2e_test.sh" "capability_enforcement_test/Cargo.toml" "capability_enforcement_test/src/lib.rs"; do
    if [ -f "$file" ]; then
        echo "✓ $file exists"
    else
        echo "✗ $file missing"
        exit 1
    fi
done

echo ""
echo "✓ All required files present"
echo ""
echo "To run the full test:"
echo "  nix run ./tests/e2e"
echo ""
echo "Or run directly with proper tools:"
echo "  cd tests/e2e && bash run_e2e_test.sh"