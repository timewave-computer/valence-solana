#!/bin/bash
set -e

# This script runs tests for off-chain components using system cargo
# Requirements:
#   - System Rust toolchain (rustup or system package)
#   - Can use latest Rust edition (including Edition 2024)

echo "=== Running Off-Chain Tests ==="
echo "Using system cargo: $(which cargo || echo 'NOT FOUND')"
echo ""

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found in PATH"
    echo "Please install Rust using rustup: https://rustup.rs/"
    exit 1
fi

# Run tests excluding on-chain programs
echo "Running tests for off-chain components..."
cargo test --workspace --exclude valence-kernel

echo ""
echo "Tests completed!"