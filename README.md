# Valence Protocol SVM

Valence is a unified development environment for building trust-minimized cross-chain DeFi applications. This Solana implementation features ZK coprocessor integration for trustless cross-chain state verification and pogram execution.

## Architecture

```
programs
├── authorization    # Permissioning with ZK support
├── processor        # Message execution engine with priority queues
├── registry         # Library and ZK program registry
├── zk_verifier      # Proof verification
├── base_account     # Token custody and vault operations
├── storage_account  # Key-value data storage and persistence
└── libraries        # Collection of utility libraries
```

## Nix Environment

The Nix environment provides a complete, reproducible Solana development setup with custom crate2nix derivations that enables incremental cached builds for various Rust toolchains, Solana CLI tools, Anchor framework, platform tools for SBF compilation, and all necessary dependencies. All packages are pinned to specific versions and automatically configured to work together.

## Quick Start

Enter Nix development environment:

```bash
nix develop
```

Set up Solana tools (first time only):
```bash
nix run .#setup-solana
```

### Development Workflow

```bash
nix run .#build                  # Build with crate2nix + Anchor (recommended)
nix run .#build-fast             # Fast incremental build (crate2nix only)
nix run .#build-crate [name]     # Build individual crate with crate2nix
nix run .#test [crate]           # Run tests (optionally specify crate)
nix run .#generate-idls          # Generate IDLs with nightly Rust
```

### Environment Management

```bash
nix run .#env-info               # Show environment status
nix run .#setup-solana           # Set up Solana unified node environment
nix run .#clear-cache            # Clear build caches (Valence, Cargo, Anchor, Nix)
```

### Deployment

```bash
nix run .#deploy [network]       # Deploy to devnet/mainnet
```

### Solana Node Environment

This flake packages the complete Solana development environment with all tools automatically configured to work together:

```bash
# Solana CLI tools
solana                           # Main Solana CLI
solana-keygen                    # Key generation utility
solana-test-validator            # Local test validator
# ... and other Solana CLI tools

# Platform tools (custom Rust/Clang toolchain for SBF compilation)
cargo-build-sbf                  # Build Solana programs with integrated platform tools
rustc                           # Rust compiler (SBF-enabled)
clang                           # Clang compiler (SBF-enabled)
```