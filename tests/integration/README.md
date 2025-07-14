# Valence Integration Tests

Integration and unit tests for the Valence Protocol, following the standard Valence project structure.

## Directory Structure

```
tests/integration/
├── shard/               # Test-specific shard helpers and utilities
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs       # Shard test configuration and helpers
├── functions/           # Test functions for integration testing
│   └── test_function/   # Simple function for testing protocol features
├── tests/               # Test files and utilities
│   ├── test_utils.rs    # Shared test utilities (validator, deployment)
│   ├── e2e_test.rs      # End-to-end protocol flow test
│   └── valence_test.rs  # Unit tests for core components
├── keypairs/            # Deterministic keypairs for testing
│   ├── registry-keypair.json
│   ├── shard-keypair.json
│   └── test_function-keypair.json
└── Cargo.toml
```

## Running Tests

```bash
# Build programs first
nix develop -c bash scripts/build-with-keys.sh

# Run all tests
nix develop -c cargo test -p valence-tests

# Run specific test
nix develop -c cargo test -p valence-tests test_end_to_end_flow
```

## Test Requirements

- Must run inside `nix develop` environment
- Programs must be built before running e2e tests
- Tests use deterministic keypairs from `keypairs/` directory

## Test Coverage

### e2e_test.rs
Complete protocol flow test that validates:
- Local validator setup
- Program deployment (Registry, Shard, Test Function)
- Function registration
- Session creation with capabilities
- Function execution
- Session consumption (linear type semantics)

### valence_test.rs
Unit tests for core components:
- Content hashing algorithms
- Capability system and bitmap operations
- PDA (Program Derived Address) derivation
- Account size calculations
- Session state hash updates

### test_utils.rs
Shared utilities for tests:
- LocalValidator: Manages test validator lifecycle
- DeployedPrograms: Handles program deployment
- Helper functions for common test operations

## Test Modules

### shard/
Contains test-specific helpers and utilities for shard operations:
- `TestShardConfig`: Configuration for test sessions with default capabilities
- `create_test_session_params()`: Helper to create standard test session parameters
- Re-exports all shard types for convenient test usage

### functions/
Contains Anchor programs used by integration tests:
- **test_function**: A simple compute unit consumer that demonstrates function execution within the Valence protocol