#!/bin/bash
set -e

echo "=== Shard Encapsulation and Capability Tests ==="
echo
echo "These tests validate that:"
echo "1. Shards are fully encapsulated and cannot directly access external programs"
echo "2. All external interactions must go through registered functions"
echo "3. Functions must have proper capabilities to perform external operations"
echo "4. Capability enforcement prevents unauthorized access"
echo

# Build all programs
echo "Building programs..."
cargo build-sbf --manifest-path ../../programs/registry/Cargo.toml
cargo build-sbf --manifest-path ../../programs/shard/Cargo.toml
cargo build-sbf --manifest-path ./Cargo.toml

# Run the tests
echo
echo "Running encapsulation tests..."
cargo test --manifest-path ./Cargo.toml -- --nocapture

echo
echo "=== Test Summary ==="
echo "✅ Shards cannot directly access external state"
echo "✅ Functions without capabilities cannot access external resources"
echo "✅ READ capability allows read operations only"
echo "✅ WRITE capability required for state modifications"
echo "✅ Multiple capabilities can be required and enforced"
echo "✅ TRANSFER capability required for token operations"
echo "✅ Malicious functions cannot bypass capability checks"
echo "✅ Capabilities are properly normalized"
echo
echo "All encapsulation tests passed! Shards are properly isolated."