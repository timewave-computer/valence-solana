# Valence E2E Tests

This directory contains end-to-end tests that demonstrate the complete integration of the Valence ecosystem, testing the full user journey from deployment to execution.

## Overview

The E2E test suite validates the entire Valence ecosystem by:
1. **Program Deployment**: Deploys all core programs (kernel, functions, test shard)
2. **Function Registry**: Sets up and validates the function registry system
3. **Shard Integration**: Deploys a test shard that integrates with the kernel
4. **Runtime Management**: Uses the runtime crate to manage session lifecycle
5. **Operation Execution**: Tests both direct operations and batch function execution
6. **State Validation**: Verifies correct state transitions and data consistency

## Structure

```
e2e/
├── test-shard/         # Test shard program that integrates with kernel
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs      # Shard implementation
├── runtime-integration-test/  # E2E test runner with runtime integration
│   ├── Cargo.toml
│   └── src/
│       └── main.rs     # Test implementation
├── justfile            # Task runner commands
├── Anchor.toml         # Anchor configuration
└── README.md           # This file
```

## Test Shard Features

The test shard (`test-shard/`) demonstrates:
- Creating sessions through the kernel
- Registering accounts in the Account Lookup Table (ALT)
- Executing registered functions via batch operations
- Performing direct operations (SPL transfers)
- Managing session lifecycle
- Stack optimization using Boxing - All large accounts are boxed to prevent stack overflow issues

## Running the Tests

### Prerequisites

Before running the e2e tests, ensure you have:

1. **Nix with flakes enabled** - The tests run in a Nix development environment for reproducibility
2. **Sufficient disk space** - The tests start a local Solana validator
3. **Network connectivity** - For downloading dependencies

### Entering the Development Environment

All e2e test commands require the Nix development environment. Always start by entering the environment:

```bash
# From the project root (valence-solana/)
nix develop --accept-flake-config

# You should see the welcome message:
# Valence Solana Development Environment
# ======================================
```

This environment provides:
- `just` - Task runner for all commands
- `solana` - Solana CLI tools
- `anchor` - Anchor framework
- `cargo` - Rust toolchain with nightly support
- All required system dependencies

### Building Programs

The project now uses the Nix BPF builder by default for deterministic builds:

```bash
# Build all programs using Nix BPF builder (recommended)
just build

# Alternative: Use cargo directly (fallback)
just build-cargo
```

The Nix BPF builder:
- Provides reproducible builds across different environments
- Automatically handles the `__client_accounts_crate.rs` stub required by Anchor
- Ensures consistent toolchain versions via zero.nix integration
- Outputs programs to `target/deploy/` for compatibility

### Quick Start

**Option 1: Using Just Commands (Recommended)**
```bash
# First, enter the Nix environment from project root
cd /path/to/valence-solana
nix develop --accept-flake-config

# Then navigate to e2e directory and run tests
cd e2e
just test              # Run complete e2e test suite with Nix-built programs
just test-debug        # Run with debug logging
just test-interactive  # Keep validator running after tests
```

**Option 2: Manual Setup**
```bash
# Always start by entering Nix environment from project root
nix develop --accept-flake-config

# Build all programs with Nix
just build

# Or build individual programs
nix build .#valence-kernel --out-link target/nix-kernel
nix build .#valence-functions --out-link target/nix-functions

# Run tests
cd e2e && just test
```

**Option 3: Development Mode**
```bash
# Enter Nix environment first
nix develop --accept-flake-config

# Navigate to e2e directory
cd e2e

# Run tests with different modes
just test-debug        # With debug output
just test-interactive  # Keep validator running
```

### What the Test Does

1. **Program Deployment**: Deploys kernel, functions, and test shard to local validator
2. **Initialization**: Initializes kernel shard and CPI allowlist
3. **Token Setup**: Creates SPL token mint and accounts for testing
4. **Session Creation**: Uses runtime to create a session with proper configuration
5. **Function Execution**: Calls registered function (ID: 2000) through batch operations
6. **Direct Transfer**: Performs direct SPL transfer using kernel's optimized path

## Test Output

A successful test run displays detailed progress through each step:

```
=== Valence E2E Test Runner ===
Step 1: Building programs
✓ valence-kernel built successfully
✓ valence-functions built successfully  
✓ test-shard built successfully

Step 2: Starting local Solana validator
✓ Validator started (PID: 12345)
✓ Validator is ready!

Step 3: Setting up test wallet
✓ Wallet balance: 100 SOL

Step 4: Building E2E test
✓ E2E test compiled

Step 5: Running E2E test
=== Valence E2E Test ===
Connected to Solana cluster at http://localhost:8899
Step 1: Deploying programs...
Step 2: Initializing kernel...
Step 3: Initializing test shard...
Step 4: Setting up token accounts...
Step 5: Creating session with runtime...
Step 6: Executing registered function...
Step 7: Executing direct transfer...
=== E2E Test Completed Successfully! ===
```

## Troubleshooting

### Common Issues

**"command not found: just" error:**
```bash
# You're not in the Nix environment. Enter it first:
nix develop --accept-flake-config
```

**Test hangs during validator startup:**
```bash
# Kill any existing validators
pkill solana-test-validator
# Clear test ledger and retry
rm -rf e2e/.test-ledger
```

**Build failures:**
```bash
# Ensure you're in the correct Nix environment
echo $IN_NIX_SHELL  # Should not be empty

# If Nix BPF builder fails, check for missing stub files
ls programs/*/src/__client_accounts_crate.rs

# Try cleaning and rebuilding
just clean && just build

# Fall back to cargo if needed
just build-cargo
```

**Program deployment errors:**
```bash
# Check program sizes aren't too large
ls -la target/deploy/
# Verify programs compiled correctly
solana program show <program-id> --url http://localhost:8899
```

### Debug Mode

For detailed logging during development:
```bash
# In Nix environment
cd e2e
RUST_LOG=debug,solana_test_validator=info just test
```

### Manual Debugging

If you need to debug interactively:
```bash
# Start validator manually
solana-test-validator --reset --quiet &
# Run individual test components
cd tests && cargo run --bin e2e-test
```

### Local Development Workflow
```bash
# Enter Nix environment first
nix develop --accept-flake-config

# Quick test during development
cd e2e && just test

# Run tests with debug output
cd e2e && just test-debug

# Keep validator running after tests for debugging
cd e2e && just test-interactive

# Run specific test components
cd e2e/runtime-integration-test && cargo test

# Continuous testing during development
cd e2e && watch -n 30 'just test'
```