#!/usr/bin/env bash
set -e

# This script builds programs with specific keypair IDs for testing

# Cleanup function to restore original source files
cleanup() {
    if [ -f programs/registry/src/lib.rs.bak ]; then
        echo "Restoring original registry source..."
        mv programs/registry/src/lib.rs.bak programs/registry/src/lib.rs 2>/dev/null || true
    fi
    if [ -f programs/shard/src/lib.rs.bak ]; then
        echo "Restoring original shard source..."
        mv programs/shard/src/lib.rs.bak programs/shard/src/lib.rs 2>/dev/null || true
    fi
    if [ -f tests/integration/functions/test_function/src/lib.rs.bak ]; then
        echo "Restoring original test_function source..."
        mv tests/integration/functions/test_function/src/lib.rs.bak tests/integration/functions/test_function/src/lib.rs 2>/dev/null || true
    fi
}

# Set up trap to ensure cleanup runs on script exit
trap cleanup EXIT INT TERM

REGISTRY_KEYPAIR="${1:-tests/integration/keypairs/registry-keypair.json}"
SHARD_KEYPAIR="${2:-tests/integration/keypairs/shard-keypair.json}"
TEST_FUNCTION_KEYPAIR="${3:-tests/integration/keypairs/test_function-keypair.json}"

# Get the public keys
REGISTRY_PUBKEY=$(solana-keygen pubkey "$REGISTRY_KEYPAIR")
SHARD_PUBKEY=$(solana-keygen pubkey "$SHARD_KEYPAIR")
TEST_FUNCTION_PUBKEY=$(solana-keygen pubkey "$TEST_FUNCTION_KEYPAIR")

echo "Building with program IDs:"
echo "  Registry: $REGISTRY_PUBKEY"
echo "  Shard: $SHARD_PUBKEY"
echo "  Test Function: $TEST_FUNCTION_PUBKEY"

# Update the declare_id! in the programs
sed -i.bak "s/declare_id!(\".*\");/declare_id!(\"$REGISTRY_PUBKEY\");/" programs/registry/src/lib.rs
sed -i.bak "s/declare_id!(\".*\");/declare_id!(\"$SHARD_PUBKEY\");/" programs/shard/src/lib.rs
sed -i.bak "s/declare_id!(\".*\");/declare_id!(\"$TEST_FUNCTION_PUBKEY\");/" tests/integration/functions/test_function/src/lib.rs

# Build the programs
echo "Building registry..."
cargo build-sbf --manifest-path programs/registry/Cargo.toml

echo "Building shard..."
cargo build-sbf --manifest-path programs/shard/Cargo.toml

echo "Building test function..."
cargo build-sbf --manifest-path tests/integration/functions/test_function/Cargo.toml

echo "Build complete!"
echo "Original source files restored."