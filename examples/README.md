# Valence Examples

This directory contains working examples demonstrating key Valence concepts and functionality.

## Examples

### `direct_operations.rs`
**Purpose**: Demonstrates direct operations for simple, single-purpose tasks like token transfers and session management.

**Key Features**:
- Session-based authorization patterns
- Account Lookup Table management
- Session invalidation for ownership transfer
- Optimized execution paths that bypass batch overhead

### `batch_operations.rs`
**Purpose**: Shows complex batch operations for atomic multi-step execution in DeFi flows.

**Key Features**:
- Complex liquidation flow patterns
- Governance proposal execution
- Dynamic account resolution at runtime
- Atomic execution of multiple operations

### `zk-transfer-limit/`
**Purpose**: Demonstrates privacy-preserving transfer limits using zero-knowledge proofs.

**Key Features**:
- ZK proof generation and verification
- Privacy-preserving compliance checking
- Integration with Valence's ZK verification function
- Demonstrates regulatory compliance without data exposure

## Running the Examples

All examples can be run using Nix from the examples directory:

```bash
# Navigate to examples directory
cd examples/

# Run individual examples
nix develop ../ --accept-flake-config -c cargo run --bin direct_operations
nix develop ../ --accept-flake-config -c cargo run --bin batch_operations

# For the ZK transfer limit example
cd zk-transfer-limit/
nix develop ../../ --accept-flake-config -c cargo run --bin zk-transfer-limit

# Build all examples
nix develop ../ --accept-flake-config -c cargo build

# Build Solana programs with Nix (if examples include programs)
# From project root: just build
```

## Architecture Integration

These examples demonstrate integration with Valence's core components:

- **Sessions**: Secure execution contexts with namespace organization
- **Batch Operations**: Atomic multi-step execution with current limits (12 accounts, 5 operations)
- **Account Lookup Tables**: Pre-registered accounts for efficient access
- **Function Registry**: Integration with hardcoded function registry (e.g., ZK verification ID: 1000)
- **Guard Accounts**: Security policy enforcement and access control

## Development Notes

- Examples use the current valence-kernel architecture with simplified, hardcoded registries
- All examples compile and run successfully with educational output
- ZK example demonstrates concept integration without requiring actual cryptographic libraries
- Examples serve as templates for building real Valence applications 