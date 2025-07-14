# Scripts Directory

This directory contains utility scripts for the Valence Solana project.

## Scripts

### `build-with-keys.sh`
**Purpose**: Builds programs with specific keypair IDs for deterministic addresses.

**Usage**: 
```bash
./scripts/build-with-keys.sh
```

**What it does**:
- Uses keypairs from `tests/integration/keypairs/` for deterministic program addresses
- Temporarily updates `declare_id!` macros during build
- Builds all three programs: Registry, Shard, Test Function
- Restores original IDs after building to keep git clean

### `run-benchmarks.sh`
**Purpose**: Runs performance benchmarks for Valence Protocol optimizations.

**Usage**:
```bash
./scripts/run-benchmarks.sh
```

**What it does**:
- Builds the project in release mode
- Runs performance benchmarks from `tests/performance_benchmarks.rs`
- Shows performance improvements for:
  - Capability checking
  - State hash computation
  - Session validation
  - Account lookups

### `test-build.sh`
**Purpose**: Tests building all workspace components using the nix development environment.

**Usage**: 
```bash
./scripts/test-build.sh
```

**What it does**:
- Tests building workspace root, programs, SDK, services, and tests
- Uses `nix develop` to ensure proper environment
- Provides detailed success/failure reporting
- Validates all components compile correctly

### `verify-workspace.sh`
**Purpose**: Verifies the workspace structure and configuration.

**Usage**:
```bash
./scripts/verify-workspace.sh
```

**What it does**:
- Validates that expected workspace members exist
- Checks for old/deprecated references
- Verifies Anchor version consistency
- Ensures proper workspace configuration

### `validate-session.sh`
**Purpose**: Validates that the Session implementation is complete and working.

**Usage**:
```bash
./scripts/validate-session.sh
```

**What it does**:
- Checks that all Session API components are implemented
- Validates capability system and bitmap implementation
- Verifies documentation and examples exist
- Tests compilation of core components
- Confirms all Session features are working

## Running Scripts

All scripts should be run from the project root directory:

```bash
# From project root
./scripts/build-with-keys.sh
./scripts/run-benchmarks.sh
./scripts/test-build.sh
./scripts/verify-workspace.sh
./scripts/validate-session.sh
```

## Development Notes

- These scripts use nix for consistent environments
- They include color output for better readability
- All scripts are designed to be CI-friendly
- Scripts follow the project preference for organizing utilities in the scripts directory