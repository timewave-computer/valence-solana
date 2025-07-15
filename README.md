# Valence Protocol

Implementation of the Valence Protocol for Solana.

## Project Structure

```
valence-solana/
├── programs/
│   ├── common/          # Shared security utilities
│   ├── registry/        # Function registry (singleton)
│   └── shard/           # Session management
├── sdk/                 # Rust SDK
├── examples/            # Usage examples
└── tests/integration/   # End-to-end tests
```

## Architecture

```
┌─────────────────┐
│   Client SDK    │
└────────┬────────┘
         │
┌────────▼────────┐
│     Shard       │ (User programs with sessions)
└────────┬────────┘
         │ Direct CPI
┌────────▼────────┐
│    Registry     │ (Singleton service)
└─────────────────┘
```

## Quick Start

1. **Build programs:**
```bash
./build.sh
```

2. **Deploy locally:**
```bash
solana-test-validator
solana program deploy target/deploy/registry.so
solana program deploy target/deploy/shard.so
```

3. **Run example:**
```bash
cargo run --example simple_usage
```

## Key Features

### Security
- All program IDs validated
- Bounds checking throughout
- Content verification for functions
- Linear session consumption (UTXO-like)

### Performance
- O(1) capability checking with bitmaps
- O(1) account lookups via HashMap indexing
- Single-pass account validation
- Efficient SHA-256 state hashing

### Developer Experience
- SDK with fluent API
- Simple session management
- Type-safe capabilities
- Error handling

## Example Usage

```rust
use valence_sdk::{ValenceClient, SessionBuilder};

// Register a function
let content_hash = client
    .register_function(program_id, metadata, bytecode_hash)
    .await?;

// Create a session
let session = SessionBuilder::new()
    .with_read()
    .with_write()
    .with_execute()
    .build(&client)
    .await?;

// Execute function
client.execute_function(
    session,
    program_id,
    metadata,
    bytecode_hash,
    input_data,
).await?;

// Consume session
client.consume_session(session).await?;
```

## Implementation Status

Registry program with content verification  
Shard program with session management  
Real function execution via CPI  
Cryptographic state hashing  
Security utilities and validation  
Minimal SDK with clean abstractions  
End-to-end test suite  
