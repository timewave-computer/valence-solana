# Valence SDK

A Rust SDK for interacting with Valence protocol, providing a clean, ergonomic interface for session management and kernel operations.

## Features

- **Client Management**: High-level client for connecting to Valence protocol
- **Session Operations**: Create and manage session accounts with namespace support
- **Move Semantics**: Rust-like ownership semantics for account borrowing
- **Compute Optimization**: Built-in compute unit estimation and batching
- **Type Safety**: Full type safety with comprehensive error handling

## Quick Start

```rust
use valence_sdk::*;
use anchor_client::Cluster;
use solana_sdk::signature::Keypair;
use std::rc::Rc;

// Create client
let payer = Rc::new(Keypair::new());
let client = ValenceClient::new(Cluster::Devnet, payer, None)?;

// Create session
let session_params = CreateSessionParams {
    namespace: "my-app".to_string(),
    // ... other params
};
let session = client.create_session(session_params).await?;

// Use session for operations
session.borrow_account(account_pubkey, AccessMode::Write).await?;
// ... perform operations
session.release_account(account_pubkey).await?;
```

## Modules

- `client` - Main client and connection management
- `session` - Session creation and management
- `compute` - Compute unit estimation and optimization
- `move_semantics` - Account borrowing with ownership semantics
- `error` - Comprehensive error types
