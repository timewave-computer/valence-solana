# Valence Protocol SVM

Valence is a unified development environment for building trust-minimized cross-chain DeFi applications. This Solana implementation features a diff-based functional architecture with ZK coprocessor integration for trustless cross-chain state verification and program execution.

## Architecture

The Valence Protocol implements a diff-based functional microkernel architecture with 8 core programs:

```
programs/
├── entrypoint/                  # Singleton request router with diff-based execution
├── session_factory/             # Event-driven session creation with queue system
├── base_account/                # Basic account management for sessions
├── storage_account/             # Enhanced storage capabilities
├── processor/                   # Message queue processing
├── authorization/               # Authorization and callback system
├── zk_verifier/                 # Zero-knowledge proof verification
├── registry/                    # Library and ZK program registry
├── diff/                        # Core diff system with content-addressed diffs
└── libraries/                   # Reusable libraries
    └── token_transfer/          # Token transfer library
```

### Core Programs

- **Entrypoint**: Singleton router that directs execution requests to appropriate Eval programs with diff-based processing
- **Session Factory**: Event-driven session creation with comprehensive queue system and off-chain service integration
- **Base Account**: Manages basic account functionality with library approval system
- **Storage Account**: Provides enhanced storage capabilities with batch operations
- **Processor**: Handles message queuing and processing with pause/resume functionality
- **Authorization**: Manages authorizations with callback system for cross-program communication
- **ZK Verifier**: Verifies zero-knowledge proofs with registry integration
- **Registry**: Manages libraries and ZK programs with version control and dependency resolution
- **Diff**: Core diff system implementing content-addressed diffs with atomic verification

### Diff-Based Functional Architecture

The system implements a pure functional architecture where:

- **Pure Functions**: All operations are side-effect free verification functions
- **Content-Addressed Diffs**: State changes are represented as immutable diffs
- **Atomic Verification**: All-or-nothing diff processing with rollback capability
- **Object Scoping**: Functions access only authorized objects through namespace isolation
- **Extensible Strategies**: Pluggable diff strategies for different use cases

### Event-Driven Session Factory

The session factory implements a comprehensive event-driven system:

- **Event Emission**: 9 event types for complete lifecycle tracking
- **Queue System**: Permissionless FIFO queue with deadline enforcement
- **Off-Chain Integration**: Seamless coordination with session builder services
- **Two-Phase Creation**: Support for both immediate and queued session creation
- **Batch Processing**: Efficient batch operations for high-volume scenarios

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

### Testing

```bash
nix run .#test                   # Run all tests
nix run .#test-diff              # Test diff system
nix run .#test-session-factory   # Test session factory
nix run .#test-integration       # Run integration tests
```

### Runtime Services

```bash
nix run .#session-builder        # Run session builder service
nix run .#service-health         # Check service health
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
rustc                            # Rust compiler (SBF-enabled)
clang                            # Clang compiler (SBF-enabled)
```

## Architecture Features

### Diff-Based Processing
- **Pure Functions**: All operations are deterministic and side-effect free
- **Content Addressing**: Diffs are addressed by their content hash
- **Atomic Operations**: All-or-nothing state transitions with rollback
- **Immutable History**: Complete audit trail of all state changes

### Event-Driven Design
- **Comprehensive Events**: 9 event types covering complete session lifecycle
- **Off-Chain Coordination**: Seamless integration with external services
- **Service Monitoring**: Health checks and metrics for production deployments
- **Retry Logic**: Exponential backoff and failure recovery

### Security Model
- **Namespace Isolation**: Functions access only authorized objects
- **Verification Composition**: Composable security through pure functions
- **Immutable Functions**: Verification functions cannot be modified
- **Execution Tracking**: Every operation recorded with unique execution ID

### Performance Optimizations
- **Batch Processing**: Efficient bulk operations
- **Compute Budget**: Optimized for Solana's compute constraints
- **Caching**: Verification functions cached by content hash
- **Parallel Processing**: Queue operations support parallel execution

## Documentation

Comprehensive documentation is available in the `docs/` directory:

- **Architecture**: Complete system design documentation
- **API Reference**: Detailed instruction and account documentation
- **Service Integration**: Off-chain service development guide
- **Testing Guide**: Comprehensive testing strategies
- **Deployment Guide**: Production deployment procedures

## Contributing

The project follows functional programming principles with emphasis on:

- **Pure Functions**: All operations should be deterministic
- **Immutable Data**: State changes through diffs, not mutations
- **Composability**: Small, focused functions that combine well
- **Testability**: Comprehensive test coverage for all components
- **Documentation**: Clear documentation for all public interfaces
