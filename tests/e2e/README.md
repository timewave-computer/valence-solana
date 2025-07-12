# Valence E2E Tests

This directory contains end-to-end tests for the Valence Solana framework.

## Running Tests

The e2e tests run in a completely isolated environment using Nix. All test functionality is consolidated in a single `flake.nix` file.

### Available Test Commands

```bash
# Run full e2e test with mock program deployment
nix run ./tests/e2e

# Run simple validator test (just tests validator startup and deployment)
nix run ./tests/e2e#simple

# Or from the e2e directory
cd tests/e2e
nix run         # Full test
nix run .#simple # Simple test
```

### What the Tests Do

**Full Test (`nix run`):**
- Creates a temporary isolated directory
- Copies test files and template project
- Starts a Solana validator
- Sets up PostgreSQL database for services
- Builds and starts off-chain services:
  - Session Builder Service (monitors account requests)
  - Lifecycle Manager Service (orchestrates account/session lifecycle)
- Deploys on-chain programs (Gateway, Registry, Verifier, Shard)
- Tests the complete deployment workflow
- Demonstrates off-chain service integration
- Validates API endpoints and service health
- Tests service monitoring and auto-progression

**Simple Test (`nix run .#simple`):**
- Minimal test that just verifies:
  - Validator can start
  - CLI can be configured
  - Program deployment workflow works

## Architecture

The consolidated `flake.nix` contains:
- **BPF Builder**: Declarative function to build Solana programs using zero.nix tools
- **Test Program**: Pre-built program derivation using the BPF builder
- **Valence Programs**: Pre-built Gateway, Registry, Verifier, and Shard programs
- **Off-chain Services**: Built and configured for testing
  - **Session Builder**: Monitors on-chain account requests and initializes them
  - **Lifecycle Manager**: Orchestrates complete account/session lifecycle
- **Database Setup**: PostgreSQL instance for lifecycle manager persistence
- **Simple Test**: Minimal validator and deployment test
- **Full Test Runner**: Complete e2e test using the test script with service integration

## Template Project Structure

The `capability_enforcement_test/` demonstrates proper Valence Session V2 architecture:
- `src/lib.rs` - Main entrypoint with Session V2 API usage
- `src/test_capabilities.rs` - Capability enforcement testing for Session V2
- `src/client.rs` - Client demonstrating Session V2 lifecycle
- `src/functions/` - Example functions for testing capability enforcement

The test validates:
- Session V2 capability enforcement
- Bitmap-based permission checking
- Direct session execution
- Off-chain service integration and monitoring
- API endpoint functionality
- Service health checks and metrics
- Lifecycle orchestration and auto-progression

## Off-chain Service Components

### Session Builder Service
- Monitors on-chain account requests
- Automatically initializes requested accounts
- Handles retry logic for failed initializations
- Provides monitoring of account creation workflow

### Lifecycle Manager Service
- Orchestrates complete account/session lifecycle
- Manages session state transitions
- Handles UTXO-style session consumption
- Provides REST API for lifecycle management
- Supports auto-progression of sessions
- Maintains persistence via PostgreSQL

### Service Integration Testing
The e2e test demonstrates:
- Service startup and health monitoring
- API endpoint accessibility and response validation
- Account request processing workflow
- Session lifecycle orchestration
- Service configuration and metrics collection
- Cross-service communication patterns

## Test Artifacts

Test artifacts are preserved in:
- Normal run: `tests/e2e/target/e2e-test/valence-e2e-test/`
- Isolated run: `$TEMP_DIR/target/e2e-test/valence-e2e-test/`

The test output will show the exact artifact location.

## Troubleshooting

If tests fail, the output will include:
- Validator logs from the test run
- Build logs from cargo-build-sbf
- Deployment and transaction information
- Off-chain service logs (session_builder.log, lifecycle_manager.log)
- PostgreSQL setup and connection logs
- API endpoint test results and responses

Service logs are available in the test workspace:
- `session_builder.log` - Session builder service output
- `lifecycle_manager.log` - Lifecycle manager service output  
- `postgres.log` - Database server logs
- `validator.log` - Solana validator output

Note: The tests use Solana 2.0 tools from the parent flake, ensuring consistency with the main development environment. Services are automatically cleaned up on test completion.