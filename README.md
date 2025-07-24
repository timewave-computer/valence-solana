# Valence - Minimal Secure Microkernel for Solana

A minimal microkernel providing mechanisms, not policies, for secure program execution on Solana. Valence implements session types for DeFi protocol choreography with verifier-based authorization delegation.

## Quick Start

```bash
# Clone repository
git clone https://github.com/timewave-computer/valence-solana
cd valence-solana

# Enter development environment
nix develop

# Build programs
cargo build-sbf

# Run tests
cargo test
```

## Architecture

Valence implements a microkernel architecture where the core provides fundamental mechanisms while all policies live in user-deployed verifier programs.

### Core Components (`valence-core`)

The kernel provides essential mechanisms that build upon each other:

**1. Shards** - Developer entry point for protocol logic
- Deploy your protocol's business logic as executable code
- Code integrity verified through cryptographic hashing
- Execution requires authorization from session accounts
- Provides isolated environment for protocol-specific operations

**2. Accounts** - Building blocks for authorization
- Fundamental unit of authorization in the system
- Each account specifies a verifier program that controls its usage
- Lifecycle management through usage counts and time-based expiration
- 64 bytes of metadata for storing protocol-specific state
- Nonce-based replay protection ensures operations execute exactly once

**3. Sessions** - Orchestration containers with shared context
- Container that manages collections of accounts (up to 16)
- Provides 256 bytes of shared verification data accessible to all verifiers
- Enforces linear type semantics through move operations (transfer ownership once)
- Supports hierarchical composition for complex multi-protocol operations
- Enables atomic operations across all accounts in the session

**4. Verifiers** - External authorization policies
- User-deployed programs that implement authorization logic
- Called via CPI when accounts are used
- Have read access to account data and shared session context
- Return success/failure to allow or deny operations
- Enable policy innovation without modifying the kernel

### Extensions (`valence-extensions`)

Optional utilities and patterns:
- **Math**: Fixed-point arithmetic (64.64 representation)
- **Events**: Structured event emission helpers
- **Batching**: Transaction batching patterns
- **Example Verifiers**: Reference implementations (owner, linear lending, curve)

### How It Works

1. **Deploy Verifiers**: Implement your authorization policies as separate programs
2. **Deploy Shard**: Upload your protocol logic that will execute operations
3. **Create Session**: Initialize a container for managing related accounts
4. **Add Accounts**: Create accounts within the session, each linked to a verifier
5. **Execute Shard**: Run your protocol logic with account authorization
6. **Atomic Operations**: Session automatically ensures all-or-nothing execution

The microkernel never makes authorization decisions - all policies are implemented in verifiers. This separation enables innovation in user space while maintaining a stable, minimal core.

See [Architecture Documentation](docs/architecture.md) for detailed design

## Design Principles

1. **Mechanisms, not policies** - Core provides building blocks
2. **Zero-copy by default** - All state uses fixed-size fields
3. **Minimal dynamic allocations** - Limited to session account list
4. **Single responsibility** - Each instruction does one thing
5. **Composition over inheritance** - Build complexity in userspace

## Usage

1. Deploy a verifier program (see example verifiers in `programs/valence-extensions/src/examples/`)
2. Create a session with your verifier
3. Deploy shard code for your logic
4. Execute shards using sessions for authorization

## Extensions

Optional features available as a library:
- `math` - Fixed-point arithmetic operations
- `events` - Structured event emission
- `batching` - Atomic batch execution

Enable features in Cargo.toml:
```toml
valence-extensions = { version = "0.1", features = ["math", "events"] }
```

## Examples

### Verifier Examples (in `programs/valence-extensions/src/examples/`)
- `owner_verifier.rs` - Simple owner-only verifier
- `linear_lending_verifier.rs` - Linear type enforcement for lending
- `multidimensional_curve_verifier.rs` - Complex curve verification

### Integration Examples (in `examples/`)
- `composable_sessions.rs` - Demonstrates session composition patterns
- `lending_with_voucher.rs` - Example lending protocol with voucher system

## Build

```bash
# Using Nix development shell
nix develop -c cargo build-sbf

# Or using the provided build script
nix run .#build
```

For more build options, see the development environment documentation.

## Documentation

- [Architecture Guide](docs/architecture.md) - System design and concepts
- [Developer Guide](docs/developer-guide.md) - Building with Valence
- [Security Model](docs/security-model.md) - Security analysis and best practices