# Runtime Services

This directory contains off-chain services for the Valence Protocol, specifically designed to handle events emitted by on-chain programs and provide reliable account creation and initialization services.

## Overview

The runtime services implement the **off-chain account creation pattern** where:
1. On-chain programs emit `PDAComputedEvent` with account specifications
2. Off-chain services listen for these events and create accounts
3. Services perform verification and retry logic for reliability
4. Queue operations enable efficient batch processing

## Session Builder Service

The primary service for handling session account creation based on events from the `session_factory` program.

### Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Solana RPC    │    │ Session Builder │    │    Metrics      │
│   WebSocket     │◄───│    Service      │───►│   Prometheus    │
│                 │    │                 │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                              │
                              ▼
                       ┌─────────────────┐
                       │  Account        │
                       │  Creation       │
                       │  & Verification │
                       └─────────────────┘
```

### Event Schema

The service handles comprehensive events from the session_factory program:

#### Core Events for Account Creation

1. **PDAComputedEvent** (Required)
   - Triggers off-chain account creation
   - Contains account specifications (size, owner, PDA)
   - Session metadata for initialization

2. **SessionQueuedEvent** (Monitoring)
   - Session added to initialization queue
   - Queue position and deadline information

3. **QueueProcessedEvent** (Analytics)
   - Batch processing statistics
   - Performance metrics and success rates

#### Lifecycle Events for Monitoring

4. **SessionReservedEvent** - Two-phase creation reservations
5. **SessionCreatedEvent** - Complete session creation with attestation
6. **SessionActivatedEvent** - Session activation confirmation
7. **SessionInitializedFromQueueEvent** - Queue-based initialization
8. **ManualSessionInitializedEvent** - Manual fallback initialization
9. **EmergencySessionResetEvent** - Emergency state recovery

### Required Service Behavior

#### Event Processing Requirements

```rust
// Maximum processing delays by event type
PDAComputedEvent: 5 seconds          // Critical path
SessionQueuedEvent: 1 second         // Monitoring only
SessionReservedEvent: 2 seconds      // Reservation tracking
QueueProcessedEvent: 500ms           // Analytics only
```

#### Retry Logic Requirements

- **Max Retries**: 5 attempts per operation
- **Initial Delay**: 100ms
- **Max Delay**: 30 seconds
- **Backoff**: Exponential with 2.0 multiplier
- **Failure Handling**: Log errors, update metrics, continue processing

#### Health Check Requirements

- **Interval**: 30 seconds
- **RPC Connection**: Verify slot retrieval
- **Wallet Balance**: Check service account balance
- **Event Stream**: Validate WebSocket connection
- **Response Time**: < 1 second for health endpoints

#### Required Metrics

```
accounts_created_total            Counter
accounts_failed_total             Counter
event_processing_duration        Histogram
rpc_connection_status            Gauge
service_uptime_seconds           Counter
queue_size                       Gauge
wallet_balance_lamports          Gauge
```

### Service Configuration

```toml
# config.toml
[rpc]
url = "https://api.mainnet-beta.solana.com"
commitment = "confirmed"

[service]
max_concurrent_creations = 10
keypair_path = "service-keypair.json"

[retry]
max_retries = 5
initial_delay_ms = 100
max_delay_ms = 30000

[monitoring]
health_check_interval_secs = 30
metrics_port = 3001
enable_prometheus = true
```

### API Endpoints

The service exposes the following endpoints:

#### Health Check
```
GET /health
Response: {"status": "healthy", "timestamp": "2024-01-01T00:00:00Z"}
```

#### Service Statistics
```
GET /stats
Response: {
  "accounts_created": 1234,
  "accounts_failed": 5,
  "is_running": true,
  "uptime_seconds": 86400
}
```

#### Prometheus Metrics
```
GET /metrics
Response: Prometheus format metrics
```

### Running the Service

#### With Nix (Recommended)
```bash
# Run with default configuration
nix run .#session-builder

# Run with custom config and metrics
nix run .#session-builder -- --config config.toml --enable-metrics --metrics-port 3001
```

#### Direct Cargo
```bash
cd runtime_services/session_builder
cargo run -- --enable-metrics --log-level info
```

### Event Monitoring Integration

The service integrates with Solana's WebSocket API to monitor events:

```rust
// Event subscription configuration
let filter = RpcTransactionLogsFilter::Mentions(vec![session_factory_program_id]);
let config = RpcTransactionLogsConfig {
    commitment: Some(CommitmentConfig::confirmed()),
};
```

### Error Handling

The service implements comprehensive error handling:

1. **Connection Errors**: Automatic reconnection with backoff
2. **Account Creation Failures**: Retry with exponential backoff
3. **Verification Failures**: Log and continue processing
4. **Event Parsing Errors**: Log invalid events, continue stream
5. **Resource Exhaustion**: Semaphore-based concurrency control

### Production Deployment

#### Required Environment

- **Solana RPC Access**: High-performance RPC endpoint (Triton, QuickNode, etc.)
- **Service Wallet**: Funded wallet for account creation rent
- **Monitoring**: Prometheus/Grafana for metrics collection
- **Logging**: Structured logging with appropriate log levels

#### Security Considerations

- **Private Key Management**: Secure storage of service keypair
- **Network Security**: VPN or private networking for RPC access
- **Rate Limiting**: Monitor RPC usage and rate limits
- **Wallet Monitoring**: Alert on low balance conditions

#### Scaling Considerations

- **Concurrency**: Adjust `max_concurrent_creations` based on RPC limits
- **Memory Usage**: Monitor for memory leaks in long-running deployments
- **Disk Space**: Log rotation and cleanup policies
- **Network Bandwidth**: Monitor WebSocket connection stability

### Integration Examples

#### Event Processing Handler
```rust
async fn handle_pda_computed_event(event: PDAComputedEvent) -> Result<()> {
    // 1. Validate event data
    validate_event(&event)?;
    
    // 2. Check if account already exists
    if account_exists(&event.expected_pda).await? {
        return Ok(()); // Skip duplicate
    }
    
    // 3. Create account with retry logic
    let signature = create_account_with_retry(&event).await?;
    
    // 4. Verify account creation
    verify_created_account(&event).await?;
    
    // 5. Update metrics
    metrics.record_success();
    
    Ok(())
}
```

#### Custom Event Handler
```rust
// Extend the service to handle additional events
impl SessionBuilder {
    async fn handle_custom_event(&self, event: CustomEvent) -> Result<()> {
        // Custom business logic
        match event.event_type {
            CustomEventType::SessionReserved => {
                // Handle reservation logic
                self.process_reservation(event).await?;
            }
            CustomEventType::QueueProcessed => {
                // Update analytics
                self.update_queue_analytics(event).await?;
            }
        }
        Ok(())
    }
}
```

## Development

### Building
```bash
nix develop
cd runtime_services/session_builder
cargo build --release
```

### Testing
```bash
cargo test
```

### Linting
```bash
cargo clippy -- -D warnings
cargo fmt
```

## Architecture Notes

This service implements the **Event-Driven Off-Chain Pattern** where:

1. **On-Chain Minimal**: Programs emit events with specifications
2. **Off-Chain Processing**: Services handle complex operations
3. **Verification**: Strong consistency through verification
4. **Reliability**: Retry logic and error handling
5. **Observability**: Comprehensive metrics and logging

This pattern provides:
- **Gas Efficiency**: Expensive operations moved off-chain
- **Reliability**: Retry logic and error recovery
- **Scalability**: Horizontal scaling of off-chain services
- **Monitoring**: Full observability of operations
- **Flexibility**: Easy to update service logic without on-chain changes 