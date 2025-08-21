# Valence Solana

A secure microkernel that provides fundamental mechanisms for writing secure, performant, expressive Solana programs. It offers a minimal execution environment with foundational mechanisms for building complex, stateful applications without prescribing specific authorization models. The kernel implements an on-chain linker that abstracts away Solana's account model while maintaining security through explicit account registration and capability-based access control.

At its core, Valence provides Session Accounts: isolated execution contexts that maintain their own account registries, security configurations, and hierarchical namespace organization. The system offers a hybrid execution model with optimized direct operations for common tasks and flexible batch operations for complex, atomic multi-step workflows.

The microkernel achieves security through simplicity: fixed-size data structures, explicit account pre-registration, transaction-level atomicity, and an opt-in `allow_unregistered_cpi` option. Valence follows a "mechanisms, not policies" philosophy, providing composable security primitives that protocols can combine into their own authorization models rather than enforcing rigid patterns.

## Nix Development Commands

### Development Environments
```bash
# Enter main development shell (recommended for all development)
nix develop --accept-flake-config
# Includes: Rust, Solana CLI, Anchor, native dependencies, build tools

# Run minimal Solana validator only
nix run .#node
# Provides: Local test validator without development tools

# Launch local devnet for e2e testing (validator + programs deployed)
nix run .#local-devnet  
# Provides: Test validator + deployed programs + configuration + ready for testing
```

### Build Commands
```bash
# Build all programs using Nix BPF builder (recommended)
just build
# - Provides reproducible, deterministic builds
# - Automatically handles Anchor stub files
# - Uses consistent toolchain via zero.nix
# - Outputs to target/deploy/

# Build all workspace crates (off-chain)
cargo build

# Build Solana programs using cargo directly (fallback)
just build-cargo
# Note: individual program builds work best - workspace builds may have dependency conflicts

# Build individual programs with Nix
nix build .#valence-kernel --out-link target/nix-kernel
nix build .#valence-functions --out-link target/nix-functions

# Generate IDL files for programs
nix run .#idl-build

# Generate/update Cargo.nix for optimized builds
nix run .#generate-cargo-nix
```

### Nix BPF Builder

The project includes a custom Nix BPF builder that provides deterministic builds for Solana programs:

**Features:**
- Reproducible builds across different environments
- Automatic handling of Anchor's `__client_accounts_crate.rs` stub requirement
- Consistent toolchain versions via zero.nix integration
- Pure build environment with all dependencies pre-configured
- Automatic output to `target/deploy/` for compatibility

**Usage:**
```bash
# Build all programs (default in justfile)
just build

# Build specific programs
nix build .#valence-kernel
nix build .#valence-functions

# Use in CI/CD
nix build .#bpf-programs
```

The builder automatically creates required stub files during the build process, eliminating the need to commit generated files to version control.

### Available Tools in Nix Shell
- **solana**: Solana CLI and validator tools
- **anchor**: Anchor framework for Solana development  
- **cargo**: Rust package manager with nightly toolchain
- **crate2nix**: Generate Cargo.nix for optimized Nix builds
- **Native dependencies**: Automatically configured build tools (cmake, clang, compression libraries)
- **just**: Task runner for common development commands

The Nix environment automatically configures all required environment variables for building native dependencies and resolves system-specific build issues.

### Environment Differences
- **`nix develop`**: Full development environment with all tools and dependencies
- **`nix run .#node`**: Minimal validator-only environment (no development tools)
- **`nix run .#local-devnet`**: Local devnet for e2e testing (validator + deployed programs + configuration)

## Architecture

Valence implements a hybrid execution model with two complementary paths for different use cases, built on a foundation of session-based isolation and pre-registered account access.

### Core Components

**`programs/valence-kernel`** - The minimal execution kernel:

**1. Sessions** - Isolated execution contexts
- Maintain namespace hierarchy (e.g., "defi/lending/alice")
- Track account borrowing state with efficient bitmap
- Reference guard configuration and Account Lookup Table (ALT)
- Support clean ownership transfer through invalidation

**2. Account Lookup Table (ALT)** - Pre-registered account access
- Store up to 16 borrowable accounts with permissions
- Register up to 16 programs for CPI
- Eliminate Solana's `remaining_accounts` pattern
- Provide strong security boundaries

**3. Hybrid Execution Model** - Two first-class execution paths:
- **Direct Operations**: Optimized single-purpose instructions (spl_transfer, manage_alt, etc.)
- **Batch Operations**: Flexible execution engine for complex, dynamic operations
- Both paths are intentional design choices, not legacy artifacts

**4. Function Registry** - Hardcoded function resolution
- Map function IDs to verified program addresses
- Enable secure CPI to approved implementations
- Support extensibility within security constraints

**`programs/valence-functions`** - Core function implementations:
- Reference function implementations for common patterns
- Simplified runtime environment aligned with kernel
- Example functions for math, token validation, ZK verification

**`crates/valence-sdk`** - Client development kit:
- Session management and lifecycle
- Transaction building for both execution paths
- Move semantics support
- Compute unit optimization

**`crates/valence-registry`** - Client-side registry utilities:
- Function and shard registry management
- IDL generation for integration
- Compatibility checking

**`crates/valence-runtime`** - Off-chain coordination:
- Session runtime management
- Transaction orchestration
- Event monitoring
- Security validation

### Design

The kernel follows a "mechanisms, not policies" approach, providing:
- Provides fundamental security primitives
- Does not prescribe authorization models
- Enables protocol composition
- Maintains practical usability

See the [architecture documentation](docs/000_kernel_architecture.md) for detailed design rationale.

## Design Principles

1. **Mechanisms, not policies** - Provide primitives without prescribing usage
2. **Session-based isolation** - Each session has its own security context
3. **Pre-registered access** - Explicit account declaration for security
4. **Hybrid execution** - Direct operations for simple tasks, batch for complex flows
5. **Practical security** - Balance theoretical purity with real-world usability

### Function Examples (in `programs/valence-functions/`)
- Reference implementations for common function patterns
- Math operations, token validation, ZK verification
- Examples of kernel integration

## Development

**Workspace Structure:**
- `programs/valence-kernel` - Core execution kernel
- `programs/valence-functions` - Reference function implementations  
- `crates/valence-sdk` - Client SDK for kernel interaction
- `crates/valence-registry` - Client-side registry utilities
- `crates/valence-runtime` - Off-chain coordination service

**Key Concepts:**
- **Sessions**: Isolated execution contexts with namespaces
- **ALT**: Pre-registered account access (no `remaining_accounts`)
- **Direct Operations**: Optimized single-purpose instructions
- **Batch Operations**: Flexible execution for complex flows
- **Function Registry**: Hardcoded mapping of IDs to programs

**Generated Artifacts:**
- `target/idl/valence_kernel.json` - Program IDL for client integration
- `target/deploy/*.so` - Deployable Solana program binaries
