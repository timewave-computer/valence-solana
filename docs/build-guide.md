# Valence Solana Build Guide

## Overview

The Valence Solana project has two distinct types of components that require different build processes:

1. **On-Chain Programs** - Smart contracts that run on Solana (use `cargo build-sbf`)
2. **Off-Chain Components** - Client libraries, SDKs, and services (use regular `cargo build`)

## Why Separate Builds?

- **On-chain programs** must be compiled for the SBF (Solana Berkeley Format) target using Solana's platform tools (Rust 1.84)
- **Off-chain components** can use your system's Rust toolchain (supports Edition 2024)
- This separation allows us to use modern dependencies (like valence-coprocessor) in off-chain code without conflicts

## Build Commands

### Using Nix (Recommended)

```bash
# Build everything
nix run .#build

# Build only on-chain programs
nix run .#build-onchain

# Build only off-chain components
nix run .#build-offchain
```

### Using Scripts Directly

```bash
# Build everything
./scripts/build-all.sh

# Build only on-chain programs
./scripts/build-onchain.sh

# Build only off-chain components
./scripts/build-offchain.sh
```

### Manual Building

```bash
# On-chain programs (kernel)
cd programs/kernel
cargo build-sbf

# Off-chain SDK
cd programs/sdk
cargo build --release

# Off-chain services
cd programs/services/session_builder
cargo build --release
```

## Component Types

### On-Chain Programs
- `programs/kernel/` - The main Valence kernel program
- Built with: `cargo build-sbf`
- Output: `.so` files in `target/deploy/`
- Restrictions: No network access, no std::thread, limited dependencies

### Off-Chain Components
- `programs/sdk/` - Rust SDK for interacting with Valence
- `programs/services/session_builder/` - Service for building sessions
- Built with: regular `cargo build`
- Output: binaries in `target/release/`
- Can use: Full Rust ecosystem, including Edition 2024 dependencies

## Workspace Configuration

The Cargo workspace is configured to exclude on-chain programs from default builds:

```toml
[workspace]
members = [
    "programs/kernel",           # On-chain
    "programs/sdk",              # Off-chain
    "programs/services/session_builder",  # Off-chain
    "tests",                     # Off-chain
]

# Default members exclude on-chain programs
default-members = [
    "programs/sdk",
    "programs/services/session_builder",
    "tests",
]
```

## Dependency Management

### For On-Chain Programs
- Cannot use dependencies that require Edition 2024
- Must be compatible with Solana's platform tools (Rust 1.84)
- Should minimize dependencies

### For Off-Chain Components
- Can use any Rust edition
- Can use valence-coprocessor and other modern dependencies
- Full async runtime support (tokio, etc.)

## Build Artifacts

### On-Chain Programs
```
target/deploy/
├── valence_kernel.so      # Deployed program binary
└── valence_kernel-keypair.json  # Program keypair
```

### Off-Chain Components
```
target/release/
├── valence-cli            # SDK CLI tool
├── session_builder        # Session builder service
├── libvalence_sdk.rlib    # SDK library
└── deps/                  # Dependencies
```

## Troubleshooting

### Edition 2024 Errors
If you see errors about `edition2024`, you're likely trying to build off-chain code with on-chain tools:
- Use `cargo build` instead of `cargo build-sbf` for off-chain components
- Check that you're in the correct directory

### Missing Dependencies
For off-chain builds:
```bash
# Ensure you have protobuf compiler
brew install protobuf  # macOS
apt-get install protobuf-compiler  # Ubuntu
```

### Build Cache Issues
```bash
# Clear Solana build cache
rm -rf ~/.cache/solana/v1.48/

# Clear Cargo cache
cargo clean
```

## Development Workflow

1. **Develop on-chain programs**
   - Edit code in `programs/kernel/`
   - Build with `./scripts/build-onchain.sh`
   - Test with `solana program deploy`

2. **Develop off-chain components**
   - Edit code in `programs/sdk/` or `programs/services/`
   - Build with `./scripts/build-offchain.sh`
   - Run tests with `cargo test`

3. **Full integration testing**
   - Build everything with `./scripts/build-all.sh`
   - Deploy programs to localnet
   - Run services and test end-to-end