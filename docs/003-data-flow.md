# Data Flow and Execution Patterns

This document traces how data flows through the Valence system during common operations. Understanding these patterns is essential for building applications and debugging issues.

## Account and Session Lifecycle

Valence uses a two-tier model where individual accounts hold capabilities and state, while sessions group accounts for coordinated operations.

### Account Creation Phase

Account creation follows a two-phase pattern:

```
User ──request_account──> Shard
         │
         └──> Creates AccountRequest PDA
               - capabilities: Vec<String>
               - init_state_hash: [u8; 32]
               - owner: Pubkey
```
*Figure 1: Account request creates a PDA with requested capabilities and expected initialization state*

The user calls request_account on their shard with desired capabilities and a hash of the expected initial state. The shard creates an AccountRequest that serves as a pending account marker.

The init_state_hash provides a critical security property: the user can verify that the account will be initialized with the exact state they expect, preventing manipulation by the initialization service.

### Account Initialization Phase

Account initialization requires off-chain coordination:

```
AccountRequest ──monitor──> Account Builder Service
                               │
                               ├─> Reads capabilities
                               ├─> Builds init state
                               ├─> Verifies hash
                               │
                               └──initialize_account──> Shard
                                         │
                                         └──> Creates Account PDA
```
*Figure 2: Off-chain service monitors for requests and initializes accounts with verified state*

The account builder service:
1. Monitors for new AccountRequest accounts
2. Reads the requested capabilities
3. Constructs appropriate initialization state
4. Verifies the state hash matches the request
5. Calls initialize_account to create the active account

### Session Creation and Consumption

Sessions group multiple accounts for coordinated operations:

```
User ──create_session──> Shard
         │
         └──> Creates Session
               - accounts: Vec<Pubkey>
               - namespace: String
               - is_consumed: false

Session ──consume_session──> Shard
            │
            ├──> Marks old session consumed
            ├──> Records consumption with tx signature
            └──> Can create new sessions
```
*Figure 3: Sessions implement linear type semantics through consumption*

Sessions can only be consumed once, providing UTXO-like guarantees for state transitions.

This separation enables complex initialization logic without bloating on-chain code while maintaining security through hash verification.

## Bundle Execution

Bundles group multiple operations for atomic or sequenced execution. Valence supports two execution modes with different trade-offs.

### Synchronous Execution

Synchronous bundles execute all operations in a single transaction:

```
execute_sync_bundle(bundle)
    │
    ├─> Verify session active
    ├─> For each operation:
    │     ├─> Get function program from imports
    │     ├─> Execute via CPI
    │     ├─> Update state hash
    │     └─> Verify diff if expected
    │
    └─> Update session state hash
```
*Figure 3: Synchronous execution processes all operations atomically with immediate state updates*

The execution flow:
1. Validates the session is active
2. Iterates through operations sequentially
3. Resolves each function hash to a program
4. Invokes the function via CPI
5. Chains the result into the state hash
6. Optionally verifies against expected diff
7. Updates the final session state

All operations succeed or fail together. This mode suits small bundles that fit within transaction size limits.

### Asynchronous Execution

Asynchronous bundles span multiple transactions:

```
start_async_bundle(bundle)
    │
    └─> Create ExecutionState PDA
          - operations: Vec<Operation>
          - current_operation: 0
          - state_hash: [u8; 32]

continue_async_bundle(bundle_id)
    │
    ├─> Load ExecutionState
    ├─> Execute next N operations
    ├─> Update checkpoint
    └─> Mark complete when done
```
*Figure 4: Asynchronous execution creates persistent state for multi-transaction processing*

Asynchronous execution enables:
- Large bundles exceeding transaction limits
- Expensive operations requiring dedicated transactions
- Resumable execution after failures
- Progress tracking for long-running tasks

The execution state serves as a checkpoint, allowing any authorized party to continue execution.

## Function Invocation

Function invocation demonstrates the complete data flow through kernel components:

```
Shard ──────> Import Resolution ────> Registry Lookup ────> CPI Execution
  │                │                      │                     │
  │                ├─> Check imports      ├─> Derive PDA       ├─> Build accounts
  │                ├─> Get hash           ├─> Read program     ├─> Forward data
  │                └─> Check policy       └─> Return ID        └─> Return result
  │
  └─> Handle Result
       ├─> Update state hash
       ├─> Verify diff
       └─> Continue execution
```
*Figure 5: Function invocation flows from shard through import resolution to actual execution*

Key steps in function invocation:

1. **Import Resolution**: Check if function is imported and whether to respect deregistration
2. **Registry Lookup**: Derive PDA from hash and read program ID
3. **CPI Construction**: Build instruction with function arguments
4. **Execution**: Invoke function program and capture result
5. **State Update**: Incorporate result into state hash chain

## State Transition Verification

State transitions use hash chaining for verification:

```
Previous Hash ──┐
                ├──> SHA256 ──> New Hash
Operation Data ─┘
```
*Figure 6: State transitions chain previous hash with operation data to produce deterministic new state*

Each operation in a bundle produces a new state hash by combining:
- Previous state hash (32 bytes)
- Operation result data (variable length)

This creates an immutable audit trail of state transitions. Expected diffs can be provided to verify execution followed the anticipated path.

## Capability Propagation

Capabilities flow from sessions through execution contexts:

```
Session                    Execution Context
  │                              │
  ├─> capabilities: ["mint"]     ├─> Check: has_capability("mint")?
  │                              ├─> Yes: Allow operation
  └─> state_hash: [...]         └─> No: Reject operation
```
*Figure 7: Capabilities from sessions gate operations during execution*

The execution context:
1. Receives capabilities from the active session
2. Validates operations against granted capabilities
3. Prevents unauthorized operations
4. Maintains capability isolation between sessions

## Error Propagation

Errors bubble up through the call stack with full context:

```
Function Program ──Error──> Shard ──Wrapped Error──> User
       │                      │                        │
       └─> Original error     └─> Add context         └─> Full trace
```
*Figure 8: Errors propagate with increasing context for debugging*

Error handling preserves:
- Original error source
- Intermediate context
- Full call stack trace

This aids debugging while maintaining abstraction boundaries.

## Performance Patterns

Understanding data flow enables performance optimization:

### Batch Operations

Group related operations in single bundles to amortize transaction costs. Each transaction has fixed overhead regardless of operation count.

### Cache Function Lookups

Function hash-to-program mappings are immutable once registered. Clients can cache lookups indefinitely, eliminating repeated registry reads.

### Parallel Session Initialization

The session builder service can process multiple requests in parallel since each initialization is independent.

### Checkpoint Strategy

For async bundles, checkpoint after expensive operations to minimize re-execution costs on failure.

## Security Considerations

Data flow patterns enforce several security properties:

1. **Hash Verification**: State hashes prevent tampering with initialization data
2. **Capability Boundaries**: Sessions limit operation scope
3. **Import Policies**: Shards control function availability
4. **Atomic Bundles**: Synchronous execution prevents partial state updates
5. **Checkpoint Integrity**: Async state tracks progress immutably

Understanding these flows helps identify security boundaries and design secure applications.