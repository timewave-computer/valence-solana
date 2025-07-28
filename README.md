# Valence - Session-Based Orchestration Protocol for Solana

A session-based orchestration protocol providing secure mechanisms for multi-program coordination on Solana. Valence implements a microkernel architecture with programmable authorization through sessions, guards, and structured operations.

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
# Build and run all Solana programs
nix run .#default

# Generate IDL files for programs
nix run .#idl-build

# Generate/update Cargo.nix for optimized builds (creates if missing, updates if exists)
nix run .#generate-cargo-nix
```

### Available Tools in Nix Shell
- **solana**: Solana CLI and validator tools
- **anchor**: Anchor framework for Solana development  
- **cargo**: Rust package manager with nightly toolchain
- **crate2nix**: Generate Cargo.nix for optimized Nix builds
- **Native dependencies**: Automatically configured build tools (cmake, clang, compression libraries)

The Nix environment automatically configures all required environment variables for building native dependencies and resolves system-specific build issues.

### Environment Differences
- **`nix develop`**: Full development environment with all tools and dependencies
- **`nix run .#node`**: Minimal validator-only environment (no development tools)
- **`nix run .#local-devnet`**: Local devnet for e2e testing (validator + deployed programs + configuration)

## Architecture

Valence implements a session-based architecture where the core implementation provides fundamental mechanisms for secure multi-program coordination.

### Core Components

**`programs/valence-kernel`** - Microkernel providing fundamental mechanisms:

**1. Sessions** - Orchestration containers for complex operations
- Manage collections of borrowed accounts from other programs
- Provide 256 bytes of shared context accessible to all guards
- Support atomic execution of multi-step operations
- Enable secure cross-program coordination

**2. Guards** - Dual authorization system
- **Built-in Guard VM**: Stack-based virtual machine with opcodes for common authorization patterns
- **External Guards**: CPI to user-deployed programs for custom authorization logic
- **Guard Compilation**: High-level guard expressions compiled to VM bytecode
- Access to session context and borrowed account data

**3. Operations** - Structured execution primitives
- **BorrowAccount**: Securely borrow accounts from other programs
- **InvokeProgram**: Execute CPIs with guard authorization
- **Custom**: Protocol-specific operations
- All operations are guard-protected and auditable

**4. Registry Integration** - Function and protocol discovery
- Cryptographic verification of function integrity
- Decentralized registry for protocol composition
- Dependency resolution and compatibility checking

**`programs/valence-functions`** - Protocol utilities and patterns:
- **Protocol Trait System**: Standardized interfaces for protocol development
- **Guard Library**: Reference implementations (multisig, timelock, state machine)
- **Function Composition**: Tools for building complex protocols from simple functions
- **Runtime Context**: Environment abstractions for off-chain orchestration

**`crates/valence-sdk`** - Client development kit:
- Session management and lifecycle
- Guard compilation from high-level expressions
- Transaction building and submission
- Registry integration utilities

**`crates/valence-runtime`** - Off-chain orchestration service:
- Complex workflow management
- Multi-step operation coordination
- Event monitoring and response
- Integration with external systems

### How It Works

1. **Create Session**: Initialize an orchestration container
2. **Compile Guards**: Define authorization logic using guard expressions or VM bytecode
3. **Borrow Accounts**: Securely access accounts from other programs within the session
4. **Execute Operations**: Run guard-protected operations atomically
5. **Registry Integration**: Discover and compose functions from the decentralized registry

The core never makes authorization decisions - all policies are implemented in guards. This separation enables innovation while maintaining security and auditability.

See documentation in `docs/` for detailed guides on [sessions](docs/sessions.md), [guards](docs/guards.md), and [registry integration](examples/registry-workflow/).

## Design Principles

1. **Session-based orchestration** - Operations grouped in atomic containers
2. **Dual authorization model** - Built-in VM + external guard programs
3. **Zero-copy operations** - Efficient account borrowing without data copying
4. **Composable functions** - Build complex protocols from simple primitives
5. **Registry-driven discovery** - Decentralized function and protocol registry

## Usage

### Basic Session Workflow

```rust
// 1. Create a session
let session = valence_kernel::Session::new(session_params)?;

// 2. Define guards (VM bytecode or external program)
let guard = Guard::vm(guard_bytecode);

// 3. Borrow accounts with guard protection
session.borrow_account(account_pubkey, guard)?;

// 4. Execute operations atomically
session.execute_session_operations(operations)?;
```

### Registry Integration

```rust
// Discover functions from registry
let functions = registry.search(FunctionQuery {
    tags: vec!["defi", "lending"],
    audit: Some("verified"),
})?;

// Compose protocol from registry functions
let protocol = ProtocolBuilder::new()
    .add_function(functions.lending_v1)
    .add_function(functions.oracle_v2)
    .build()?;
```

## Examples

### Core Examples (in `examples/`)
- [`registry-workflow/`](examples/registry-workflow/) - Complete registry integration workflow showcasing protocol composition and function reuse
- `simple/` - Basic session and guard examples
- `atomic-operations/` - Multi-step atomic operation patterns
- `lending-protocol/` - Example lending protocol implementation

### Guard Examples (in `programs/valence-functions/`)
- Built-in VM guards with opcodes for common patterns
- External guard programs for custom authorization logic
- Guard composition and compilation examples

## Build

```bash
# Build all workspace crates (recommended)
cargo build

# Build Solana programs for deployment
cargo build-sbf

# Build using Nix (includes all optimizations)
nix run .#default

# Generate IDL files for programs
nix run .#idl-build
```

**Note**: Always use the Nix development environment (`nix develop --accept-flake-config`) for building, as it provides all required system dependencies and environment variables.

## Documentation

- [Sessions Guide](docs/sessions.md) - Session-based orchestration concepts
- [Guards Guide](docs/guards.md) - Authorization system and guard VM
- [SDK Documentation](crates/valence-sdk/docs/guard_compilation.md) - Guard compilation and client development
- [Registry Workflow](examples/registry-workflow/) - Complete integration example

## Development

**Workspace Structure:**
- `programs/valence-kernel` - Main orchestration program
- `programs/valence-functions` - Protocol utilities and guard library  
- `crates/valence-sdk` - Client development kit
- `crates/valence-registry` - Function and protocol registry
- `crates/valence-runtime` - Off-chain orchestration service
- `examples/` - Integration examples and workflows

**Generated Artifacts:**
- `target/idl/valence_kernel.json` - Program IDL for client integration
- `target/deploy/*.so` - Deployable Solana program binaries
- `Cargo.nix` - Reproducible Nix build configuration