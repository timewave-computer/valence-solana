#!/bin/bash

echo "Running valence-core tests..."
echo ""

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found. Please install Rust and Cargo."
    exit 1
fi

# Run unit tests
echo "Running unit tests..."
cargo test --lib --features test-sbf

echo ""
echo "Running integration tests..."
cargo test --test "*" --features test-sbf

echo ""
echo "Test run complete."