#!/bin/bash
set -e

echo "Building Valence Solana programs..."

# Build only the kernel program which doesn't have problematic dependencies
echo "Building kernel program..."
cd programs/kernel
cargo build-sbf

echo "Build completed successfully!"
echo "Deployed program: ../../target/deploy/valence_kernel.so"