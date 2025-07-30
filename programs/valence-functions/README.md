# Valence Functions

A collection of reusable function implementations for the Valence protocol. This crate provides individual function modules that can be imported and used by shard programs within the Valence ecosystem.

## Overview

The valence-functions crate follows a simplified architecture where each function is a standalone module that can be imported directly by shard programs. This approach enables maximum flexibility and composability while maintaining clean separation of concerns.

### Design Philosophy

- **Individual Functions**: Each file contains one function that can be imported independently
- **Direct Imports**: Shard programs import functions directly without complex trait systems
- **Minimal Dependencies**: Functions have minimal external dependencies for maximum portability
- **Clear Interfaces**: Each function has well-defined inputs, outputs, and error handling

## Available Functions

### Core Functions

- **`identity`** - Identity function for testing and validation
- **`math_add`** - Safe integer addition with overflow protection
- **`zk_verify`** - Zero-knowledge proof verification with multiple proof system support
- **`token_validate`** - SPL token validation and metadata verification

## Function Structure

Each function follows this pattern:

```rust
// functions/example.rs
use anchor_lang::prelude::*;
use crate::error::FunctionError;

/// Function implementation
pub fn example_function(input: ExampleInput) -> Result<ExampleOutput> {
    // Function logic here
    Ok(output)
}

/// Unique identifier for this function
pub const FUNCTION_ID: u64 = 1001;
```

## Usage in Shard Programs

```rust
// In your shard program
use valence_functions::functions::identity;
use valence_functions::functions::math_add;

// Use functions directly
let result = identity::identity(42)?;
let sum = math_add::math_add(10, 20)?;
```

## Error Handling

All functions use the centralized `FunctionError` enum for consistent error handling:

```rust
#[error_code]
pub enum FunctionError {
    #[msg("Invalid input parameters")]
    InvalidInput,
    #[msg("Computation overflow")]
    Overflow,
    #[msg("Invalid proof data")]
    InvalidProof,
    // ... more error variants
}
```

## Adding New Functions

To add a new function:

1. Create a new file in `src/functions/your_function.rs`
2. Implement your function following the established pattern
3. Export the function in `src/functions/mod.rs`
4. Add appropriate tests
5. Update this README with the new function

## Building

```bash
# Build as Solana program using Nix (recommended)
nix build ../.#valence-functions --out-link target/nix-functions
# Or from project root: just build

# Build the functions crate (off-chain)
cargo build

# Build as Solana program using cargo (fallback)
cargo build-sbf

# Run tests
cargo test
```

## Integration

This crate is designed to work seamlessly with:
- **Valence Kernel**: Core execution environment
- **Valence SDK**: Client-side integration libraries
- **Shard Programs**: Application-specific programs using these functions

## Testing

Each function includes comprehensive tests covering:
- Valid input cases
- Edge cases and boundary conditions
- Error conditions and proper error handling
- Integration with the broader Valence ecosystem