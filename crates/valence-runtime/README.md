# Valence Runtime

Off-chain runtime service for monitoring on-chain state and orchestrating protocol flows.

## Overview

The Valence Runtime is designed as a non-custodial service that:
- Monitors on-chain state through WebSocket subscriptions
- Orchestrates complex protocol flows without holding keys
- Builds unsigned transactions for external signing
- Provides deterministic account generation
- Creates comprehensive audit trails

## Architecture

### Core Components

1. **State Monitor** (`state_monitor.rs`)
   - WebSocket subscriptions to on-chain accounts
   - Real-time state change notifications
   - Configurable account filtering

2. **Orchestrator** (`orchestrator.rs`)
   - Protocol flow definition and execution
   - Step-by-step transaction sequencing
   - Conditional execution logic
   - Retry policies and error handling

3. **Transaction Builder** (`transaction_builder.rs`)
   - Unsigned transaction construction
   - Transaction simulation
   - Compute budget optimization
   - Priority fee configuration

4. **Event Stream** (`event_stream.rs`)
   - Unified event bus for all runtime events
   - Filterable event subscriptions
   - Audit trail generation

## Usage

```rust
use valence_runtime::{Runtime, RuntimeConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Configure runtime
    let config = RuntimeConfig {
        rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
        ws_url: "wss://api.mainnet-beta.solana.com".to_string(),
        commitment: CommitmentConfig::confirmed(),
        max_retries: 3,
        enable_simulation: true,
    };
    
    // Initialize runtime
    let runtime = Runtime::new(config).await?;
    
    // Start services
    runtime.start().await?;
    
    // Subscribe to events
    let mut events = runtime.subscribe().await;
    
    // Monitor specific account
    let state_monitor = runtime.state_monitor();
    state_monitor.subscribe_account(account_pubkey, |update| {
        println!("Account updated: {:?}", update);
    }).await?;
    
    // Build transaction
    let tx_builder = runtime.transaction_builder();
    let unsigned_tx = tx_builder
        .add_instruction(instruction)
        .with_compute_units(200_000)
        .build("My transaction".to_string())
        .await?;
    
    // Unsigned transaction ready for external signing
    println!("Transaction ready for signing: {:?}", unsigned_tx);
    
    Ok(())
}
```

## Protocol Flow Example

```rust
use valence_runtime::{ProtocolFlow, FlowStep, InstructionTemplate};

// Define a lending protocol deposit flow
let deposit_flow = ProtocolFlow {
    id: "lending-deposit".to_string(),
    name: "Lending Protocol Deposit".to_string(),
    steps: vec![
        FlowStep {
            name: "approve-tokens".to_string(),
            description: "Approve token transfer".to_string(),
            instructions: vec![/* token approval instruction */],
            conditions: vec![
                Condition::BalanceGreaterThan {
                    pubkey: user_token_account,
                    lamports: deposit_amount,
                }
            ],
            on_success: Some("execute-deposit".to_string()),
            on_failure: None,
        },
        FlowStep {
            name: "execute-deposit".to_string(),
            description: "Execute deposit to lending pool".to_string(),
            instructions: vec![/* deposit instruction */],
            conditions: vec![],
            on_success: None,
            on_failure: None,
        },
    ],
    timeout: Duration::from_secs(60),
    retry_policy: RetryPolicy::default(),
};

// Register and execute flow
orchestrator.register_flow(deposit_flow).await?;
let instance_id = orchestrator.start_flow(
    "lending-deposit".to_string(),
    context,
).await?;
```

## Security Considerations

1. **No Key Management**: The runtime never holds or manages private keys
2. **Transaction Simulation**: All transactions are simulated before submission
3. **Audit Trails**: Every operation is logged with full context
4. **Deterministic Addresses**: All PDAs are derived deterministically
5. **External Signing**: Transactions must be signed externally

## Configuration

The runtime can be configured through environment variables:

```bash
VALENCE_RPC_URL=https://api.mainnet-beta.solana.com
VALENCE_WS_URL=wss://api.mainnet-beta.solana.com
VALENCE_COMMITMENT=confirmed
VALENCE_MAX_RETRIES=3
VALENCE_ENABLE_SIMULATION=true
```

## Development

### Running Tests

```bash
cargo test -p valence-runtime
```

### Building

```bash
cargo build -p valence-runtime --release
```

## License

Apache-2.0