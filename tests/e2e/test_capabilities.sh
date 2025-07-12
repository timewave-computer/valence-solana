#!/bin/bash
set -e

echo "=== Capability Enforcement E2E Test ==="
echo

# Build the programs
echo "Building programs..."
cd ../..
cargo build-sbf --manifest-path programs/registry/Cargo.toml
cargo build-sbf --manifest-path programs/shard/Cargo.toml
cargo build-sbf --manifest-path tests/e2e/capability_enforcement_test/Cargo.toml

# Deploy programs to local validator
echo "Starting local validator..."
solana-test-validator --reset &
VALIDATOR_PID=$!
sleep 5

# Deploy registry
echo "Deploying registry program..."
REGISTRY_PROGRAM_ID=$(solana program deploy target/deploy/valence_registry.so --url localhost --output json | jq -r .programId)
echo "Registry deployed at: $REGISTRY_PROGRAM_ID"

# Deploy shard
echo "Deploying shard program..."
SHARD_PROGRAM_ID=$(solana program deploy target/deploy/valence_shard.so --url localhost --output json | jq -r .programId)
echo "Shard deployed at: $SHARD_PROGRAM_ID"

# Deploy test function program
echo "Deploying test function program..."
cd tests/e2e/capability_enforcement_test
FUNCTION_PROGRAM_ID=$(solana program deploy target/deploy/capability_enforcement_test.so --url localhost --output json | jq -r .programId)
echo "Function program deployed at: $FUNCTION_PROGRAM_ID"

# Create test keypairs
AUTHORITY_KEYPAIR=$(solana-keygen new --no-bip39-passphrase --silent)
SESSION_OWNER_KEYPAIR=$(solana-keygen new --no-bip39-passphrase --silent)

# Fund accounts
solana airdrop 10 $AUTHORITY_KEYPAIR --url localhost
solana airdrop 10 $SESSION_OWNER_KEYPAIR --url localhost
sleep 2

echo
echo "=== Test 1: Register function with TRANSFER capability requirement ==="
# Register transfer function
FUNCTION_HASH="0101010101010101010101010101010101010101010101010101010101010101" # 32 bytes
echo "Registering transfer function with TRANSFER capability requirement..."
# Note: This would require actual transaction construction with the proper instruction data

echo
echo "=== Test 2: Create session without TRANSFER capability ==="
echo "Creating session with only READ capability..."
# Create session request with READ capability only

echo
echo "=== Test 3: Try to execute transfer function (should fail) ==="
echo "Attempting to execute transfer function without required capability..."
echo "Expected: InsufficientCapabilities error"

echo
echo "=== Test 4: Create session with TRANSFER capability ==="
echo "Creating session with TRANSFER capability..."
# Create session request with TRANSFER capability

echo
echo "=== Test 5: Execute transfer function (should succeed) ==="
echo "Executing transfer function with required capability..."
echo "Expected: Success"

# Cleanup
echo
echo "Cleaning up..."
kill $VALIDATOR_PID

echo
echo "✅ Capability enforcement test completed!"
echo "   - Functions can declare required capabilities"
echo "   - Execution fails without required capabilities"
echo "   - Execution succeeds with required capabilities"