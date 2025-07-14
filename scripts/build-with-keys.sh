#!/usr/bin/env bash
set -e

# This script builds programs with specific keypair IDs for testing

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

# Restore original IDs (optional - for cleaner git status)
mv programs/registry/src/lib.rs.bak programs/registry/src/lib.rs
mv programs/shard/src/lib.rs.bak programs/shard/src/lib.rs
mv tests/integration/functions/test_function/src/lib.rs.bak tests/integration/functions/test_function/src/lib.rs

echo "Build complete!"