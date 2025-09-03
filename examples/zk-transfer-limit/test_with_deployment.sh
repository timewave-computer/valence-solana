#!/usr/bin/env bash
# Simple test runner that demonstrates ZK Transfer Limit with nix deployment
# This shows how to use the sophisticated nix flake system for integration testing

set -e

echo "=== ZK Transfer Limit Integration Test ==="
echo "Using nix flake system for program deployment"
echo ""

# Run tests first to make sure everything builds
echo "1. Running unit tests..."
nix develop --command cargo test -- --skip test_zk_transfer_with_move_semantics
echo "✓ Unit tests passed"
echo ""

# Run the binary to show ZK transfer limit functionality
echo "2. Running ZK transfer limit demonstration..."
nix develop --command cargo run --release
echo "✓ ZK transfer limit demo completed"
echo ""

# Run integration test (shows deployment-aware testing)
echo "3. Running integration test..."
nix develop --command cargo test test_zk_transfer_with_move_semantics -- --nocapture
echo "✓ Integration test completed"
echo ""

echo "=== Test Summary ==="
echo "✓ Unit tests verify ZK proof generation and move semantics"
echo "✓ Binary demonstrates complete ZK transfer workflow"
echo "✓ Integration test shows deployment-aware testing structure"
echo ""
echo "To run with actual deployed programs, use:"
echo "  nix run .#local-devnet  # Start local devnet with deployed programs"
echo "  # Then run integration test with TEST_RPC_URL=http://localhost:8899"
echo ""
echo "The nix flake system provides:"
echo "  - Automatic program building (nix run .#default)"
echo "  - Local validator with deployment (nix run .#local-devnet)"  
echo "  - Complete development environment (nix develop)"
echo ""