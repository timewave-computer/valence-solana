# Valence Protocol SVM

Valence is a capability-based microkernel for building secure cross-chain applications on Solana. It features a clean Session API that abstracts away infrastructure complexity, enabling developers to focus on application logic while maintaining strong security guarantees.

## Architecture

Valence implements a capability-based microkernel architecture with 4 core programs:

```
programs/
├── gateway/                     # Request router and entry point
├── registry/                    # Function registry and lookup
├── verifier/                    # Verification predicate routing  
└── shard/                       # User-deployed application logic
```

### Microkernel Components

- **Gateway**: Thin routing layer that forwards requests to appropriate programs using Cross-Program Invocation (CPI)
- **Registry**: Singleton managing the global function registry with content-addressed function lookup
- **Verifier**: Routes verification predicates to specialized verifier programs for pluggable verification strategies
- **Shard**: User-deployed programs implementing application logic using Sessions for state management

### Session V2 API

The primary developer interface is the **Session V2 API**, which provides a clean abstraction:

```rust
// Create sessions with capabilities
let mut capabilities = Capabilities::none();
capabilities.add(Capability::Read);
capabilities.add(Capability::Write);
capabilities.add(Capability::Transfer);

let session = create_session_v2(
    ctx, capabilities.0, initial_state, namespace, nonce, metadata
)?;

// Execute operations directly
execute_on_session(ctx, function_hash, args)?;

// Execute atomic bundles
execute_bundle_v2(ctx, SimpleBundle { session, operations })?;
```

### Key Features

#### Capability-Based Security
- **Bitmap Capabilities**: O(1) permission checks using efficient bitmaps
- **Pre-Aggregated Permissions**: Capabilities computed at session creation, not runtime
- **Automatic Enforcement**: Capability checking handled transparently by the system
- **Fine-Grained Control**: Granular permissions for different operation types

#### Developer Experience
- **Single Abstraction**: Developers only work with Sessions, no account complexity
- **Simple API**: Three main functions cover all use cases
- **Automatic State Management**: State aggregation and synchronization handled internally
- **Clear Error Messages**: Helpful error messages for capability violations

#### Performance Optimizations
- **100x Faster Capability Checking**: Bitmap operations vs string parsing
- **Direct Execution**: No registry lookups during operation execution
- **Reduced Memory Usage**: 40% reduction compared to string-based approaches
- **Optimized Bundles**: Efficient atomic operation execution

## Nix Environment

The Nix environment provides a complete, reproducible Solana development setup with custom crate2nix derivations that enables incremental cached builds for various Rust toolchains, Solana CLI tools, Anchor framework, platform tools for SBF compilation, and all necessary dependencies. All packages are pinned to specific versions and automatically configured to work together.

### BPF Builder

The project includes a declarative BPF builder that uses zero.nix tools to build Solana programs deterministically:

```nix
# Build a BPF program declaratively
valence-shard = buildBPFProgram {
  name = "valence-shard";
  src = ./.;
  cargoToml = "programs/shard/Cargo.toml";
};
```

This eliminates the need for manual `cargo build-sbf` commands and ensures reproducible builds across environments. The builder automatically configures the proper Rust toolchain, platform tools, and build environment using the comprehensive Solana toolset from zero.nix.

## Quick Start

### Environment Setup

Enter Nix development environment:
```bash
nix develop
```

Build all programs:
```bash
nix run .#build
```

Validate Session V2 implementation:
```bash
./scripts/validate-session-v2.sh
```

### Session V2 Development

Create your first Session-based application:

```rust
use valence_shard::{*, Capability, Capabilities};

#[program]
pub mod my_app {
    use super::*;

    pub fn create_app_session(
        ctx: Context<CreateAppSession>,
        app_data: Vec<u8>,
    ) -> Result<()> {
        // Define capabilities your app needs
        let mut capabilities = Capabilities::none();
        capabilities.add(Capability::Read);
        capabilities.add(Capability::Write);
        capabilities.add(Capability::Transfer);

        // Create session with direct capability specification
        create_session_v2(
            ctx.accounts.session_ctx,
            capabilities.0,
            app_data,
            "my-app".to_string(),
            1,
            vec![]
        )
    }

    pub fn execute_app_operation(
        ctx: Context<ExecuteAppOperation>,
        function_hash: [u8; 32],
        args: Vec<u8>,
    ) -> Result<()> {
        // Execute operation directly on session
        // Capabilities checked automatically
        execute_on_session(ctx.accounts.session_ctx, function_hash, args)
    }
}
```

### Development Workflow

```bash
nix run .#build                  # Build with crate2nix + Anchor (recommended)
nix run .#build-fast             # Fast incremental build (crate2nix only)
nix run .#build-crate [name]     # Build individual crate with crate2nix
nix run .#build-bpf-programs     # Build all BPF programs using declarative Nix builder
nix run .#test-bpf-builder       # Test the BPF builder functionality
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

### Utility Scripts

Additional utility scripts are available in the `scripts/` directory:

```bash
./scripts/test-build.sh          # Test building all components
./scripts/verify_workspace.sh    # Verify workspace structure
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

### Microkernel Design
- **Minimal Core**: Four focused programs with clear responsibilities
- **User Space Logic**: All application logic in user-deployed shards
- **Modular Components**: Independent evolution of components
- **Pluggable Strategies**: Extensible verification and routing

### Capability-Based Security
- **Explicit Permissions**: All operations require declared capabilities
- **Bitmap Efficiency**: O(1) capability checking with 64-bit bitmaps
- **Principle of Least Privilege**: Sessions request only needed capabilities
- **Automatic Enforcement**: Runtime capability validation

### Content-Addressed Functions
- **Immutable References**: Functions identified by content hash
- **Deterministic Resolution**: Hash-based function lookup
- **Deduplication**: Identical functions share storage
- **Integrity Guarantees**: Code cannot be substituted

### Linear Type Sessions
- **UTXO-like Semantics**: Sessions consumed exactly once
- **State Atomicity**: Atomic state transitions via consumption
- **Audit Trail**: Complete history of session evolution
- **Concurrent Safety**: No double-spending of state

## Documentation

Comprehensive documentation is available in the `docs/` directory:

### For Developers
- **[Session V2 API](docs/session-v2-api.md)**: Complete API reference for the recommended developer interface
- **[Session V2 Tutorial](docs/session-v2-tutorial.md)**: Step-by-step guide to building applications
- **[Token Swap Example](examples/token_swap_v2/)**: Real-world Session V2 implementation example

### Architecture Documentation
- **[Architecture Overview](docs/001-architecture-overview.md)**: System design and principles
- **[API Reference](docs/005-api-reference.md)**: Complete instruction and account documentation
- **[Integration Guide](docs/006-integration.md)**: Service integration patterns
- **[Data Flow](docs/003-data-flow.md)**: Request routing and execution flow

### Examples and Templates
- **[Template Project](template_project/)**: Starter template for new applications
- **[E2E Tests](tests/e2e/)**: End-to-end testing examples
- **[Performance Benchmarks](tests/session_v2/performance_benchmarks.rs)**: Performance comparison tests

## Contributing

The project emphasizes clean, secure, and performant design:

- **Session-First Design**: APIs should expose Sessions, not internal complexity
- **Capability-Based Security**: All operations require explicit capabilities
- **Performance Optimization**: Favor O(1) operations and efficient data structures
- **Developer Experience**: APIs should be intuitive and well-documented
- **Comprehensive Testing**: All functionality should have corresponding tests

### Development Principles

- **Hide Complexity**: Internal account management should be invisible to developers
- **Bitmap Efficiency**: Use capability bitmaps for O(1) permission checks
- **Direct Execution**: Minimize indirection and registry lookups
- **Clear Error Messages**: Provide helpful feedback for capability violations
- **Atomic Operations**: Bundle operations should succeed or fail together

### Getting Started

1. Read the [Session V2 Tutorial](docs/session-v2-tutorial.md)
2. Check out the [Token Swap Example](examples/token_swap_v2/)
3. Run the validation script: `./scripts/validate-session-v2.sh`
4. Study the [API Reference](docs/session-v2-api.md)
5. Build something awesome with Sessions!
