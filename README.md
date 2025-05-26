# Valence Protocol for Solana

A trust-minimized cross-chain DeFi development environment with zero-knowledge proof capabilities for Solana.

## Overview

Valence Protocol enables secure, configurable cross-chain applications through a modular architecture of specialized programs. The protocol features ZK coprocessor integration for trustless cross-chain state verification and execution.

Key Features:
- ZK Integration: SP1 proof verification with Sparse Merkle Tree support
- Modular Architecture: Composable programs for authorization, processing, and verification
- Cross-Chain Ready: Built for trustless cross-chain coordination
- Solana-Optimized: Efficient compute usage and transaction batching

## Architecture

```
Core Programs:
├── Authorization Program    # Access control and message routing
├── Processor Program       # Message execution engine  
├── Registry Program        # Library and ZK program registry
├── ZK Verifier Program     # SP1 proof verification
├── Base Account Program    # Token custody and operations
└── Storage Account Program # Key-value data storage

Libraries:
└── Token Transfer Library  # Optimized token operations
```

## Quick Start

### Setup

1. Enter development environment:
   ```bash
   nix develop
   ```

2. Set up Solana tools (first time only):
   ```bash
   nix run .#setup-solana
   ```

3. Build all programs:
   ```bash
   nix run .#build
   ```

4. Run tests:
   ```bash
   nix run .#test
   ```

## Development

### Available Commands

```bash
# Development workflow
nix run .#build              # Build all programs
nix run .#test [program]     # Run tests (87 total tests)
nix run .#format             # Format code
nix run .#lint               # Lint code

# Environment management  
nix run .#env-info           # Show environment status
nix run .#setup-solana       # Set up Solana platform tools
nix run .#clear-cache        # Clear build caches

# Deployment
nix run .#deploy [network]   # Deploy to devnet/mainnet
```

### Testing

The project includes comprehensive Rust unit tests:

```bash
# Run all tests (87 total)
nix run .#test

# Run specific program tests
nix run .#test authorization  # 10 tests
nix run .#test registry      # 14 tests  
nix run .#test processor     # 19 tests
nix run .#test base_account  # 11 tests
nix run .#test zk_verifier   # 23 tests
```
