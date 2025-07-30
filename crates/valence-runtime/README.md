# Valence Runtime

Off-chain runtime service for monitoring Valence protocol state and orchestrating transaction flows. The runtime builds unsigned transactions for external signing and provides real-time state monitoring.

## Features

- **Session Management**: Track and cache valence-kernel session states
- **Transaction Building**: Construct unsigned transactions for external signing
- **State Monitoring**: WebSocket-based monitoring of on-chain account changes
- **Protocol Coordination**: Orchestrate multi-step protocol flows
- **Security Validation**: Transaction validation and security policy enforcement
- **Event Streaming**: Real-time event emission and filtering

## Architecture

The runtime is designed as a stateless service that:
- **Does not hold private keys** - builds unsigned transactions only
- **Monitors on-chain state** - tracks session and account changes
- **Coordinates workflows** - manages complex multi-transaction flows  
- **Validates transactions** - applies security policies before signing

## Modules

- `session` - Session state management and caching
- `transaction` - Transaction building and instruction construction  
- `monitoring` - WebSocket state monitoring and event streaming
- `coordination` - Protocol flow orchestration and execution
- `security` - Transaction validation, audit logging, and signing services
- `core` - Configuration and error types
- `types` - Common runtime types and utilities

## Quick Start

```rust
use valence_runtime::*;

// Create runtime configuration
let config = RuntimeConfig {
    rpc_url: "https://api.devnet.solana.com".to_string(),
    ws_url: "wss://api.devnet.solana.com".to_string(),
    commitment: CommitmentConfig::confirmed(),
    max_retries: 3,
    enable_simulation: true,
};

// Initialize runtime
let runtime = Runtime::new(config).await?;

// Start services
runtime.start().await?;

// Build transactions
let tx_builder = runtime.transaction_builder();
let unsigned_tx = tx_builder
    .add_instruction(my_instruction)
    .with_compute_units(200_000)
    .build("My transaction".to_string())
    .await?;

// Monitor session state
let session_state = runtime.load_session(session_pubkey).await?;
```

## Use Cases

- DeFi protocol automation and execution
- Multi-signature wallet coordination  
- Cross-program transaction orchestration
- Real-time protocol state monitoring
- Security policy enforcement and audit trails

Built for production use with comprehensive error handling, metrics, and observability.

