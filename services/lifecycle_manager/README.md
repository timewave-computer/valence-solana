# Valence Lifecycle Manager

A comprehensive service that manages the complete lifecycle of accounts and sessions in the Valence protocol, handling the linear type progression and orchestrating state transitions.

## Overview

The Lifecycle Manager replaces the simple session_builder service with a full-featured system that:

- **Monitors** on-chain account requests and initializes accounts
- **Tracks** session creation and consumption events
- **Orchestrates** linear progression of sessions based on rules
- **Manages** the complete lifecycle from account creation to session consumption
- **Provides** REST APIs for querying lifecycle state

## Architecture

The service consists of four main components:

### 1. Monitor
- Polls on-chain state for new events
- Detects account requests, session creations, and consumptions
- Updates database with current state
- Tracks linear progressions

### 2. Builder
- Processes pending account requests
- Builds initial state based on capabilities
- Verifies state hashes match expectations
- Initializes accounts on-chain

### 3. Orchestrator
- Evaluates progression rules
- Automatically progresses sessions when conditions are met
- Handles session consumption and creation
- Manages bundle execution for state transitions

### 4. API Server
- REST endpoints for querying lifecycle state
- Prometheus metrics for monitoring
- WebSocket support for real-time updates (future)

## Linear Type Model

The service tracks entities through their linear progression:

```
AccountRequest → Account → InSession → ConsumedSession
                    ↓
                 Session → ActiveSession → ConsumedSession
                                               ↓
                                          New Sessions
```

Each transition is recorded with:
- Previous state
- New state
- Timestamp
- Transaction signature (when applicable)

## Configuration

Environment variables:

```bash
# Solana Configuration
RPC_URL=http://localhost:8899
WS_URL=ws://localhost:8900
WALLET_PATH=~/.config/solana/id.json
SHARD_PROGRAM_ID=<shard_program_pubkey>

# Database Configuration
DATABASE_URL=postgres://localhost/valence_lifecycle

# Message Queue (for future use)
AMQP_URL=amqp://localhost:5672

# Service Configuration
API_PORT=8080
POLL_INTERVAL=5  # seconds
MAX_ACCOUNTS_PER_SESSION=10
CONSUMPTION_TIMEOUT=300  # seconds
AUTO_PROGRESS=false  # Enable automatic progression

# Logging
RUST_LOG=info
```

## Database Schema

The service uses PostgreSQL to track:

- **account_requests**: Pending account initialization requests
- **accounts**: Initialized accounts with capabilities
- **sessions**: Active and consumed sessions
- **session_consumptions**: Audit trail of session consumption
- **linear_progressions**: Current state and history of entities
- **lifecycle_events**: Event log for auditing
- **progression_rules**: Configurable rules for automatic progression

## API Endpoints

### Health Check
```
GET /health
```

### Metrics
```
GET /metrics
```
Prometheus-formatted metrics

### Account Status
```
GET /accounts/{account_id}
```
Returns linear progression for an account

### Session Status
```
GET /sessions/{session_id}
```
Returns linear progression for a session

### List Active Sessions
```
GET /sessions
```
Returns all active (non-consumed) sessions

### Get Lifecycle Events
```
GET /events?limit=100&offset=0&event_type=session_created
```
Query lifecycle events with filtering

### Create Progression Rule
```
POST /rules
Content-Type: application/json

{
  "id": "rule-1",
  "name": "Auto-consume idle sessions",
  "condition": {
    "SessionIdleFor": 300
  },
  "action": {
    "ConsumeAndCreate": [{
      "account_indices": [0, 1],
      "namespace": "recycled",
      "metadata": []
    }]
  },
  "enabled": true
}
```

## Progression Rules

Rules define automatic state transitions:

### Conditions
- `AllAccountsHaveCapability`: All accounts in session have a specific capability
- `SessionIdleFor`: Session has been idle for N seconds
- `StateHashMatches`: Session state matches a specific hash
- `CustomPredicate`: Custom condition logic

### Actions
- `ConsumeAndCreate`: Consume session and create new ones
- `ExecuteBundle`: Execute operations on the session
- `NotifyWebhook`: Send notification to external service

## Metrics

The service exposes Prometheus metrics:

- `valence_account_requests_total`: Total account requests processed
- `valence_accounts_initialized_total`: Successfully initialized accounts
- `valence_sessions_created_total`: Total sessions created
- `valence_sessions_consumed_total`: Total sessions consumed
- `valence_active_sessions`: Current active sessions
- `valence_active_accounts`: Current active accounts
- `valence_progression_rules_evaluated_total`: Rules evaluated
- `valence_progression_rules_matched_total`: Rules that matched
- `valence_account_initialization_duration_seconds`: Init time histogram
- `valence_session_lifetime_seconds`: Session lifetime histogram

## Running the Service

### Development
```bash
# Run database migrations
sqlx migrate run

# Start the service
cargo run
```

### Production
```bash
# Build release binary
cargo build --release

# Run with systemd
sudo systemctl start valence-lifecycle-manager
```

## Monitoring

### Grafana Dashboard

Import the included dashboard for visualizing:
- Account creation rate
- Session lifecycle metrics
- Rule evaluation performance
- Error rates and latencies

### Alerts

Example Prometheus alerts:
```yaml
- alert: HighAccountBacklog
  expr: valence_account_requests_total - valence_accounts_initialized_total > 100
  annotations:
    summary: "High number of pending account requests"

- alert: StaleSessions
  expr: valence_active_sessions > 1000
  annotations:
    summary: "Too many active sessions, may need cleanup"
```

## Development

### Adding New Progression Rules

1. Define condition in `types.rs`:
```rust
pub enum ProgressionCondition {
    // ... existing conditions
    MyNewCondition { threshold: u64 },
}
```

2. Implement condition check in `orchestrator.rs`:
```rust
ProgressionCondition::MyNewCondition { threshold } => {
    // Implementation
}
```

3. Create rule via API or database

### Extending the Service

The service is designed to be extensible:

- Add new event types in `monitor.rs`
- Implement custom builders in `builder.rs`
- Create new API endpoints in `api.rs`
- Add metrics in `metrics.rs`

## Security Considerations

- Account initialization verifies state hashes
- Only account owners can create sessions
- Session consumption is atomic and audited
- All state transitions are logged

## Future Enhancements

- WebSocket support for real-time updates
- Multi-shard support
- Advanced rule conditions with CEL expressions
- Batch account initialization
- Session templates and factories
- Integration with external orchestration tools