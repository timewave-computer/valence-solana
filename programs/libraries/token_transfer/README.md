# Token Transfer Library

This library provides token transfer functionality with support for SPL Token and Token-2022 standards.

## Features

- Token transfers with fee collection
- Batch transfers
- Token authority delegation
- Configuration-based allowlists
- Fee calculations with slippage tolerance

## Token Helpers

The library includes a utility module `token_helpers.rs` that provides functions for working with Token-2022:

- `get_token_program_id()`: Returns the Program ID for Token-2022
- `transfer_tokens()`: Executes a token transfer using Token-2022
- `token_account_exists()`: Validates if a token account exists

## Testing with LiteSVM

This library uses [LiteSVM](https://github.com/LiteSVM/litesvm) for testing Solana programs in Rust. LiteSVM provides a fast, lightweight simulation environment that eliminates the need for running a local validator.

### Running Tests

To run tests with the correct macOS environment settings:

```bash
# Run all token_transfer tests
./scripts/litesvm-test.sh token_transfer

# Run a specific test
./scripts/litesvm-test.sh token_transfer -- test_token_transfer_library

# Run only the token_helpers tests
./scripts/litesvm-test.sh token_transfer -- utils::token_helpers::tests
```

## Architecture

The library follows a modular design:
- `instructions/`: Contains instruction handlers for each operation
- `utils/`: Utility functions including token helpers
- `state.rs`: Defines account structures
- `lib.rs`: Entry point for the library

## Dependencies

- `anchor-lang`: ^0.29.0
- `anchor-spl`: ^0.29.0 with token and token_2022 features
- `solana-program`: 1.17.14
- `spl-token-2022`: 0.9.0 