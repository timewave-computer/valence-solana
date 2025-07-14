# Tests Directory

This directory contains tests for your Valence shard and functions.

## Test Structure

```
tests/
├── shard_test.rs         # Unit tests for shard functionality
├── integration_test.rs   # Integration tests (to be created)
└── e2e_test.rs          # End-to-end tests (to be created)
```

## Running Tests

### Unit Tests

Run all tests:
```bash
cargo test
```

Run tests for a specific package:
```bash
cargo test -p template-shard
cargo test -p hello-world-function
```

Run with output:
```bash
cargo test -- --nocapture
```

### Integration Tests

For Anchor-based integration tests, create TypeScript tests:

1. Install dependencies:
```bash
yarn install
```

2. Run tests:
```bash
anchor test
```

## Writing Tests

### 1. Unit Tests (Rust)

Unit tests go in the `src/lib.rs` files or in this directory. Example:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::*;

    #[test]
    fn test_shard_initialization() {
        // Test shard state initialization
        let authority = Pubkey::new_unique();
        // ... test logic
    }
}
```

### 2. Integration Tests (Rust)

Create `integration_test.rs`:

```rust
use anchor_lang::prelude::*;
use template_shard::*;

#[test]
fn test_function_registration_and_execution() {
    // Test registering a function
    // Test executing the function
    // Verify state changes
}
```

### 3. Anchor Tests (TypeScript)

Create `tests/shard.ts`:

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TemplateShard } from "../target/types/template_shard";

describe("template-shard", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.TemplateShard as Program<TemplateShard>;

  it("Initializes the shard", async () => {
    // Test implementation
  });

  it("Registers a function", async () => {
    // Test implementation
  });

  it("Executes a function", async () => {
    // Test implementation
  });
});
```

## Test Scenarios

### Shard Tests

1. **Initialization**
   - Shard can be initialized once
   - Authority is set correctly
   - PDA derivation works

2. **Function Registration**
   - Only authority can register
   - Function count increments
   - Duplicate prevention (when implemented)

3. **Function Execution**
   - Valid functions can be executed
   - Invalid functions are rejected
   - Input data is passed correctly

### Function Tests

1. **Hello World Function**
   - Processes empty names correctly
   - Handles various input sizes
   - Timestamp is reasonable

2. **Math Operations Function**
   - All operations work correctly
   - Overflow/underflow handling
   - Division by zero prevention

### Integration Tests

1. **Full Flow**
   - Deploy shard
   - Deploy functions
   - Register functions with shard
   - Execute functions through shard
   - Verify outputs

## Setting Up Test Environment

### Local Validator

Start a local validator for testing:
```bash
solana-test-validator
```

With specific features:
```bash
solana-test-validator \
  --reset \
  --quiet \
  --bpf-program YOUR_PROGRAM_ID target/deploy/template_shard.so
```

### Test Keypairs

Generate test keypairs:
```bash
mkdir -p tests/keypairs
solana-keygen new -o tests/keypairs/test-authority.json --no-bip39-passphrase
solana-keygen new -o tests/keypairs/test-user.json --no-bip39-passphrase
```

### Environment Setup

Create `.env.test`:
```env
ANCHOR_PROVIDER_URL=http://127.0.0.1:8899
ANCHOR_WALLET=~/.config/solana/id.json
```

## Best Practices

1. **Test Isolation**: Each test should be independent
2. **Clear Names**: Use descriptive test names
3. **Setup/Teardown**: Use proper test fixtures
4. **Error Cases**: Test both success and failure paths
5. **Edge Cases**: Test boundary conditions
6. **Documentation**: Comment complex test logic

## Debugging Tests

### Enable Logging

```rust
use env_logger;

#[test]
fn test_with_logs() {
    env_logger::init();
    // Your test
}
```

### Use Anchor's Testing Tools

```typescript
// Get transaction logs
const tx = await program.methods.initialize().rpc();
const logs = await provider.connection.getTransaction(tx, {
  commitment: "confirmed"
});
console.log(logs.meta.logMessages);
```

## Continuous Integration

Example GitHub Actions workflow:

```yaml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
      - uses: anchor-lang/anchor-action@v1
      - run: cargo test
      - run: anchor test
```