# Valence Kernel

The core execution environment for the Valence protocol on Solana. The kernel provides a minimal, secure foundation for building complex, stateful applications through session-based account management and batch operation execution.

## Overview

Valence Kernel implements an "on-chain linker" architecture that abstracts away Solana's complex account model while maintaining security through explicit account registration and capability-based access control.

### Key Features

- **Session Accounts**: Isolated execution contexts with their own account registries and security configurations
- **Batch Operations**: Atomic execution of complex operation sequences with dynamic account resolution  
- **Direct Operations**: Optimized instruction handlers for common operations like token transfers
- **Hierarchical Namespaces**: Organized session management with parent-child relationships
- **Account Lookup Tables (ALT)**: Pre-registration system that eliminates `remaining_accounts` patterns
- **Borrowing Semantics**: Safe concurrent account access with bitmap-based slot tracking

## Architecture

The kernel provides two complementary execution paths:

1. **Direct Operations** - Single-purpose instructions for well-defined operations
2. **Batch Operations** - Flexible execution engine for complex, dynamic scenarios

All operations execute within rich `ExecutionContext` environments that capture transaction metadata, session information, and temporal data.

## Core Components

- **Session Management**: Create, update, and invalidate isolated execution contexts
- **Guard Accounts**: Security policy configuration with opt-in risk controls
- **Account Lookup Tables**: Pre-approved account registries for secure access
- **Function Registry**: Hardcoded mapping of function IDs to verified program addresses
- **Namespace System**: Hierarchical organization with deterministic PDA derivation

## Usage

The kernel is designed to be used by:
- **Shard Programs**: Application-specific programs that leverage kernel infrastructure
- **SDK Integration**: Client libraries that construct transactions using kernel primitives
- **DeFi Protocols**: Financial applications requiring complex state management

## Security Model

The kernel follows a "mechanisms, not policies" philosophy:
- Provides foundational security primitives
- Allows protocols to compose these into their own authorization models
- Ensures transaction atomicity and prevents partial state corruption
- Implements capability-based access with explicit opt-in to risk

## Building

```bash
# Build the kernel program using Nix (recommended)
nix build ../.#valence-kernel --out-link target/nix-kernel
# Or from project root: just build

# Build using cargo directly (fallback)
cargo build-sbf

# Run tests
cargo test
```

## Documentation

For detailed architecture information, see [docs/000_kernel_architecture.md](../../docs/000_kernel_architecture.md).