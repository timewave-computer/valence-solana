# Scripts Directory

This directory contains utility scripts for the Valence Solana project.

## Scripts

### `test-build.sh`
**Purpose**: Tests building all workspace components using the nix development environment.

**Usage**: 
```bash
./scripts/test-build.sh
```

**What it does**:
- Tests building workspace root, programs, SDK, services, and e2e tests
- Uses `nix develop` to ensure proper environment
- Provides detailed success/failure reporting
- Checks for deprecated components (like old session_builder)

### `verify_workspace.sh`
**Purpose**: Verifies the workspace structure and configuration.

**Usage**:
```bash
./scripts/verify_workspace.sh
```

**What it does**:
- Validates that expected workspace members exist
- Checks for old/deprecated references (e.g., session_builder)
- Verifies Anchor version consistency
- Ensures proper workspace configuration

### `validate-session-v2.sh`
**Purpose**: Validates that the Session V2 implementation is complete and working.

**Usage**:
```bash
./scripts/validate-session-v2.sh
```

**What it does**:
- Checks that all Session V2 API components are implemented
- Validates capability system and bitmap implementation
- Verifies documentation and examples exist
- Tests compilation of core components
- Confirms all Session V2 features are working

## Running Scripts

All scripts should be run from the project root directory:

```bash
# From project root
./scripts/test-build.sh
./scripts/verify_workspace.sh
```

## Development Notes

- These scripts use nix for consistent environments
- They include color output for better readability
- All scripts are designed to be CI-friendly
- Scripts follow the project preference for organizing utilities in the scripts directory 