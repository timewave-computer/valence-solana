# API Reference

This document provides a comprehensive reference for all public APIs in the Valence protocol. The primary developer interface is the **Session V2 API**, which provides a clean abstraction that hides infrastructure complexity. Legacy APIs are also documented for reference.

## Gateway Program

Program ID: `11111111111111111111111111111111`

### Instructions

#### route
Routes operations to registry, verifier, or shard programs.

Parameters:
- `target: RouteTarget` - Destination for the operation
- `data: Vec<u8>` - Serialized instruction data for target

Context Accounts:
- `signer: Signer<'info>` - Transaction signer

RouteTarget Variants:
- `Registry { instruction: RegistryInstruction }` - Route to registry
- `Verifier { instruction: VerificationInstruction }` - Route to verifier
- `Shard { id: Pubkey, instruction_data: Vec<u8> }` - Route to specific shard

### Errors

- `InvalidTarget` - Target program not found or invalid
- `Unauthorized` - Signer lacks required permissions

## Session V2 API (Recommended)

The Session V2 API provides a clean, simple interface for developers. **This is the recommended API for all new applications.**

### Key Concepts

**Sessions** are the only abstraction you need to understand. They contain:
- Pre-aggregated capabilities (stored as efficient bitmaps)
- Application state (managed automatically)  
- Linear consumption semantics (UTXO-like guarantees)

### Instructions

#### create_session_v2
Creates a session with direct capability specification.

Parameters:
- `capabilities: u64` - Capability bitmap (e.g., `Capability::Read.to_mask() | Capability::Write.to_mask()`)
- `initial_state: Vec<u8>` - Initial application state (max 32 bytes)
- `namespace: String` - Session namespace (max 64 chars)
- `nonce: u64` - Session nonce for uniqueness
- `metadata: Vec<u8>` - Optional session metadata (max 256 bytes)

Context Accounts:
- `owner: Signer<'info>` - Session owner
- `session: Account<'info, Session>` - New session account
- `backing_account: Account<'info, ValenceAccount>` - Internal account (hidden from developers)
- `system_program: Program<'info, System>` - System program

Example:
```rust
let mut capabilities = Capabilities::none();
capabilities.add(Capability::Read);
capabilities.add(Capability::Write);
capabilities.add(Capability::Transfer);

create_session_v2(
    ctx,
    capabilities.0,
    b"app initial state".to_vec(),
    "my-dapp".to_string(),
    1,
    vec![]
)?;
```

#### execute_on_session
Executes a single operation directly on a session.

Parameters:
- `function_hash: [u8; 32]` - Function to execute
- `args: Vec<u8>` - Function arguments

Context Accounts:
- `executor: Signer<'info>` - Must be session owner
- `session: Account<'info, Session>` - Session to execute on

Capabilities are checked automatically. State is updated automatically.

Example:
```rust
execute_on_session(
    ctx,
    [1u8; 32], // function hash
    b"operation args".to_vec()
)?;
```

#### execute_bundle_v2
Executes multiple operations atomically on a session.

Parameters:
- `bundle: SimpleBundle` - Bundle with operations

Context Accounts:
- `executor: Signer<'info>` - Must be session owner
- `session: Account<'info, Session>` - Session to execute on

Example:
```rust
let operations = vec![
    SimpleOperation {
        function_hash: [1u8; 32],
        required_capabilities: Capability::Read.to_mask(),
        args: b"read data".to_vec(),
    },
    SimpleOperation {
        function_hash: [2u8; 32],
        required_capabilities: Capability::Write.to_mask(),
        args: b"write data".to_vec(),
    },
];

let bundle = SimpleBundle {
    session: session_id,
    operations,
};

execute_bundle_v2(ctx, bundle)?;
```

### Data Structures

#### Session (V2)
Primary developer abstraction.

Fields:
- `id: Pubkey` - Session identifier
- `owner: Pubkey` - Session owner
- `capabilities: u64` - Pre-aggregated capability bitmap
- `state_root: [u8; 32]` - Current application state
- `namespace: String` - Session namespace
- `is_consumed: bool` - Linear consumption flag
- `nonce: u64` - Session nonce
- `created_at: i64` - Creation timestamp
- `metadata: Vec<u8>` - Optional metadata

Methods:
- `has_capability(cap: Capability) -> bool` - Check single capability (O(1))
- `has_all_capabilities(&[Capability]) -> bool` - Check multiple capabilities
- `get_capabilities() -> Capabilities` - Get capabilities wrapper
- `update_state_root([u8; 32])` - Update application state
- `apply_state_diff(&[u8]) -> [u8; 32]` - Apply state change

#### SimpleBundle
Simplified bundle for clean API.

Fields:
- `session: Pubkey` - Session to execute on
- `operations: Vec<SimpleOperation>` - Operations to execute

#### SimpleOperation
Simplified operation with direct capability specification.

Fields:
- `function_hash: [u8; 32]` - Function to execute
- `required_capabilities: u64` - Required capability bitmap
- `args: Vec<u8>` - Function arguments

#### Capabilities
Bitmap wrapper for O(1) operations.

Methods:
- `none() -> Self` - Empty capabilities
- `add(&mut self, Capability)` - Add capability
- `has(&self, Capability) -> bool` - Check capability (O(1))
- `to_mask() -> u64` - Get raw bitmap

#### Capability
Efficient capability enumeration.

Variants:
- `Read` - Read operations
- `Write` - Write operations  
- `Execute` - Execute functions
- `Transfer` - Transfer assets
- `Mint` - Create tokens
- `Burn` - Destroy tokens
- `Admin` - Administrative operations
- `CreateAccount` - Create accounts
- And more...

Methods:
- `to_mask() -> u64` - Get capability bitmap
- `from_string(s: &str) -> Option<Self>` - Parse from string

### Performance Benefits

Session V2 provides significant performance improvements:
- **100x faster capability checking** - O(1) bitmap vs O(n) string matching
- **50% faster session creation** - Direct capability specification
- **25% faster execution** - No registry lookups during execution
- **40% memory reduction** - Compact bitmap vs string storage

## Legacy APIs

The following APIs are maintained for backward compatibility but are not recommended for new applications.

## Registry Program

Program ID: `11111111111111111111111111111112`

### Instructions

#### register
Registers a new function with the global registry.

Parameters:
- `hash: [u8; 32]` - Content hash of the function
- `program: Pubkey` - Program implementing the function
- `required_capabilities: Vec<String>` - Capabilities required to execute this function

Context Accounts:
- `authority: Signer<'info>` - Function authority
- `function_entry: Account<'info, FunctionEntry>` - PDA for function data
- `system_program: Program<'info, System>` - System program

Seeds: `[b"function", hash]`

#### unregister
Removes a function from the registry.

Parameters:
- `hash: [u8; 32]` - Hash of function to remove

Context Accounts:
- `authority: Signer<'info>` - Must match registration authority
- `function_entry: Account<'info, FunctionEntry>` - Function to close

### Account Structures

#### FunctionEntry
Stores registered function data.

Fields:
- `hash: [u8; 32]` - Function content hash
- `program: Pubkey` - Implementing program
- `authority: Pubkey` - Registration authority
- `required_capabilities: Vec<String>` - Capabilities required to execute function

### Errors

- `FunctionAlreadyRegistered` - Hash already registered
- `FunctionNotFound` - Hash not in registry
- `Unauthorized` - Caller not function authority

## Verifier Program

Program ID: `11111111111111111111111111111113`

### Instructions

#### register_verifier
Registers a verification program for a label.

Parameters:
- `label: String` - Semantic label for verifier
- `program: Pubkey` - Verification program

Context Accounts:
- `authority: Signer<'info>` - Verifier authority
- `verifier_entry: Account<'info, VerifierEntry>` - PDA for verifier
- `system_program: Program<'info, System>` - System program

Seeds: `[b"verifier", label.as_bytes()]`

#### update_verifier
Updates the program for an existing verifier.

Parameters:
- `label: String` - Verifier label to update
- `new_program: Pubkey` - New verification program

Context Accounts:
- `authority: Signer<'info>` - Must match registration authority
- `verifier_entry: Account<'info, VerifierEntry>` - Verifier to update

#### verify_predicate
Routes a verification request to the appropriate verifier.

Parameters:
- `label: String` - Verifier label
- `predicate_data: Vec<u8>` - Predicate to verify
- `context: Vec<u8>` - Verification context

Context Accounts:
- `caller: Signer<'info>` - Requesting account
- `verifier_entry: Account<'info, VerifierEntry>` - Verifier registry entry
- Additional accounts passed to verifier program

### Account Structures

#### VerifierEntry
Stores verifier registration data.

Fields:
- `label: String` - Semantic label
- `program: Pubkey` - Verification program
- `authority: Pubkey` - Registration authority

### Errors

- `VerifierAlreadyRegistered` - Label already in use
- `VerifierNotFound` - Label not registered
- `Unauthorized` - Caller not verifier authority
- `VerificationFailed` - Predicate evaluation failed

## Shard Program

Program ID: `11111111111111111111111111111114`

### Instructions

#### initialize
Initializes shard configuration.

Parameters:
- `max_operations_per_bundle: u16` - Maximum operations in one bundle
- `default_respect_deregistration: bool` - Default import policy

Context Accounts:
- `authority: Signer<'info>` - Shard authority
- `shard_config: Account<'info, ShardConfig>` - Configuration PDA
- `system_program: Program<'info, System>` - System program

Seeds: `[b"shard_config", authority.key().as_ref()]`

#### request_account
Creates an account request for off-chain initialization.

Parameters:
- `capabilities: Vec<String>` - Requested capabilities
- `init_state_hash: [u8; 32]` - Expected initialization state hash

Context Accounts:
- `owner: Signer<'info>` - Account owner
- `account_request: Account<'info, AccountRequest>` - Request PDA
- `system_program: Program<'info, System>` - System program

#### initialize_account
Initializes an account from a request (called by service).

Parameters:
- `request_id: Pubkey` - AccountRequest to fulfill
- `init_state_data: Vec<u8>` - Initialization state matching hash

Context Accounts:
- `initializer: Signer<'info>` - Service account
- `account_request: Account<'info, AccountRequest>` - Request to close
- `account: Account<'info, Account>` - New account PDA
- `system_program: Program<'info, System>` - System program

#### create_session
Creates a new session from multiple accounts.

Parameters:
- `accounts: Vec<Pubkey>` - Accounts to include in session
- `namespace: String` - Session namespace (max 64 chars)
- `nonce: u64` - Session nonce for uniqueness
- `metadata: Vec<u8>` - Optional session metadata

Context Accounts:
- `owner: Signer<'info>` - Must own all accounts in session
- `session: Account<'info, Session>` - New session PDA
- `system_program: Program<'info, System>` - System program

Note: Account references must be passed in remaining_accounts for validation.

#### consume_session
Consumes a session to create new sessions (UTXO-like).

Parameters:
- `new_sessions_data: Vec<(Vec<Pubkey>, String, u64, Vec<u8>)>` - Data for new sessions

Context Accounts:
- `owner: Signer<'info>` - Session owner
- `old_session: Account<'info, Session>` - Session to consume
- `consumption_record: Account<'info, SessionConsumption>` - Audit record
- `instructions_sysvar: AccountInfo<'info>` - For transaction signature
- `system_program: Program<'info, System>` - System program

#### execute_sync_bundle
Executes all bundle operations in one transaction.

Parameters:
- `bundle: Bundle` - Operations to execute

Context Accounts:
- `executor: Signer<'info>` - Must be session owner
- `session: Account<'info, Session>` - Active session
- `shard_config: Account<'info, ShardConfig>` - Shard configuration
- Additional accounts for function calls

#### start_async_bundle
Begins asynchronous bundle execution.

Parameters:
- `bundle: Bundle` - Operations to execute across transactions

Context Accounts:
- `executor: Signer<'info>` - Must be session owner
- `session: Account<'info, Session>` - Active session
- `execution_state: Account<'info, ExecutionState>` - New checkpoint PDA
- `shard_config: Account<'info, ShardConfig>` - Shard configuration
- `system_program: Program<'info, System>` - System program

#### continue_async_bundle
Continues asynchronous execution from checkpoint.

Parameters:
- `bundle_id: Pubkey` - ExecutionState to continue

Context Accounts:
- `executor: Signer<'info>` - Any authorized account
- `execution_state: Account<'info, ExecutionState>` - Checkpoint to update
- Additional accounts for function calls

#### import_function
Imports a function from the registry.

Parameters:
- `function_hash: [u8; 32]` - Function to import
- `respect_deregistration: Option<bool>` - Override default policy

Context Accounts:
- `authority: Signer<'info>` - Shard authority
- `shard_config: Account<'info, ShardConfig>` - Shard configuration
- `function_import: Account<'info, FunctionImport>` - Import PDA
- `registry_entry: UncheckedAccount<'info>` - Registry function entry
- `registry_program: UncheckedAccount<'info>` - Registry program
- `system_program: Program<'info, System>` - System program

Seeds: `[b"function_import", shard_config.key().as_ref(), &function_hash]`

#### update_import_policy
Changes deregistration policy for an import.

Parameters:
- `function_hash: [u8; 32]` - Imported function
- `respect_deregistration: bool` - New policy

Context Accounts:
- `authority: Signer<'info>` - Shard authority
- `shard_config: Account<'info, ShardConfig>` - Shard configuration
- `function_import: Account<'info, FunctionImport>` - Import to update

#### update_config
Updates shard configuration.

Parameters:
- `new_config: ShardConfig` - New configuration values

Context Accounts:
- `authority: Signer<'info>` - Shard authority
- `shard_config: Account<'info, ShardConfig>` - Configuration to update

#### set_paused
Pauses or unpauses the shard.

Parameters:
- `paused: bool` - Pause state

Context Accounts:
- `authority: Signer<'info>` - Shard authority
- `shard_config: Account<'info, ShardConfig>` - Configuration to update

### Account Structures

#### ShardConfig
Shard configuration and state.

Fields:
- `authority: Pubkey` - Administrative authority
- `is_paused: bool` - Pause state
- `max_operations_per_bundle: u16` - Bundle size limit
- `default_respect_deregistration: bool` - Import policy default

#### AccountRequest
Pending account initialization.

Fields:
- `id: Pubkey` - Request identifier
- `owner: Pubkey` - Requesting account
- `capabilities: Vec<String>` - Requested capabilities
- `init_state_hash: [u8; 32]` - Expected state hash
- `created_at: i64` - Unix timestamp

#### Account
Individual account with capabilities and state.

Fields:
- `id: Pubkey` - Account identifier
- `owner: Pubkey` - Account owner
- `capabilities: Vec<String>` - Granted capabilities
- `state_hash: [u8; 32]` - Current state hash
- `is_active: bool` - Active flag
- `created_at: i64` - Unix timestamp

#### Session
Linear type grouping multiple accounts.

Fields:
- `id: Pubkey` - Session identifier
- `owner: Pubkey` - Session owner
- `accounts: Vec<Pubkey>` - Accounts in this session
- `namespace: String` - Shared namespace
- `is_consumed: bool` - UTXO consumption flag
- `nonce: u64` - Session nonce
- `created_at: i64` - Unix timestamp
- `metadata: Vec<u8>` - Optional metadata

#### SessionConsumption
Audit record for session consumption.

Fields:
- `consumed_session: Pubkey` - Session that was consumed
- `created_sessions: Vec<Pubkey>` - New sessions created
- `transaction_signature: [u8; 64]` - Transaction signature
- `consumed_at: i64` - Unix timestamp

#### Bundle
Collection of operations to execute.

Fields:
- `operations: Vec<Operation>` - Operations to perform
- `mode: ExecutionMode` - Sync or Async
- `session: Pubkey` - Session this bundle operates within

#### Operation
Single operation within a bundle.

Fields:
- `function_hash: [u8; 32]` - Function to invoke
- `args: Vec<u8>` - Function arguments
- `expected_diff: Option<[u8; 32]>` - Optional verification
- `target_account: Option<Pubkey>` - Which account in session to target

#### ExecutionState
Checkpoint for async execution.

Fields:
- `bundle_id: Pubkey` - Execution identifier
- `current_operation: u16` - Progress counter
- `total_operations: u16` - Total operations
- `state_hash: [u8; 32]` - Current state hash
- `is_complete: bool` - Completion flag
- `operations: Vec<Operation>` - Remaining operations
- `session: Pubkey` - Associated session

#### FunctionImport
Imported function record.

Fields:
- `shard: Pubkey` - Importing shard
- `function_hash: [u8; 32]` - Imported function
- `program: Pubkey` - Cached program ID
- `respect_deregistration: bool` - Policy flag
- `imported_at: i64` - Unix timestamp

### Errors

- `InvalidSessionRequest` - Malformed session request
- `InvalidStateHash` - State hash mismatch
- `SessionNotActive` - Session inactive or not found
- `Unauthorized` - Insufficient permissions
- `InvalidBundle` - Malformed bundle data
- `TooManyOperations` - Bundle exceeds size limit
- `ShardPaused` - Shard is paused
- `FunctionNotFound` - Function not imported
- `FunctionMismatch` - Function hash mismatch
- `DiffMismatch` - Execution diff mismatch
- `ExecutionFailed` - Function execution error
- `InvalidCheckpoint` - Invalid execution state

## SDK Usage

The SDK provides builder patterns focused on the Session V2 API:

```rust
use valence_shard::{*, Capability, Capabilities};

// Create a session with capabilities
let mut capabilities = Capabilities::none();
capabilities.add(Capability::Read);
capabilities.add(Capability::Write);
capabilities.add(Capability::Transfer);

let session = create_session_v2(
    ctx,
    capabilities.0,
    b"initial app state".to_vec(),
    "my-dapp".to_string(),
    1,
    vec![]
)?;

// Execute a single operation
execute_on_session(
    ctx,
    [1u8; 32], // function hash
    b"operation args".to_vec()
)?;

// Execute multiple operations atomically
let operations = vec![
    SimpleOperation {
        function_hash: [2u8; 32],
        required_capabilities: Capability::Read.to_mask(),
        args: b"read data".to_vec(),
    },
    SimpleOperation {
        function_hash: [3u8; 32],
        required_capabilities: Capability::Write.to_mask(),
        args: b"write data".to_vec(),
    },
];

let bundle = SimpleBundle { session, operations };
execute_bundle_v2(ctx, bundle)?;
```

### Legacy SDK Usage

For backward compatibility, the old SDK patterns are still available:

```rust
use valence_sdk::{Client, builders::*};

// Register a function (still used)
let ix = register_function_instruction(
    registry_program,
    authority,
    function_hash,
    function_program,
)?;

// Legacy session creation (not recommended)
let ix = request_session_instruction(
    shard_program,
    owner,
    vec!["transfer".to_string()],
    init_state_hash,
)?;
```

## Common Patterns

### Session V2 Development

#### Quick Start
```rust
// 1. Create session with capabilities
let mut capabilities = Capabilities::none();
capabilities.add(Capability::Read);
capabilities.add(Capability::Write);

let session = create_session_v2(
    ctx, capabilities.0, initial_state, namespace, nonce, metadata
)?;

// 2. Execute operations
execute_on_session(ctx, function_hash, args)?;

// 3. Execute bundles
execute_bundle_v2(ctx, SimpleBundle { session, operations })?;
```

#### Application Development
1. Design your capability requirements
2. Create sessions with minimal required capabilities  
3. Execute operations directly on sessions
4. Use bundles for atomic multi-operation transactions
5. Handle capability errors gracefully

#### Capability Patterns
```rust
// Basic user: read and write only
let user_caps = Capability::Read.to_mask() | Capability::Write.to_mask();

// Token operations: transfer, mint, burn
let token_caps = Capability::Transfer.to_mask() | 
                Capability::Mint.to_mask() | 
                Capability::Burn.to_mask();

// Admin: all capabilities
let mut admin_caps = Capabilities::none();
admin_caps.add(Capability::Admin);
// ... add all needed capabilities
```

### Legacy Patterns (Not Recommended)

#### Deploying a Shard (Legacy)
1. Deploy shard program code
2. Call initialize with configuration
3. Import required functions
4. Ready for session requests

#### Session Usage (Legacy)
1. Request session with capabilities
2. Wait for initialization
3. Execute bundles with session
4. Session persists across bundles

### Function Registration (Still Used)
1. Deploy function program
2. Calculate program hash
3. Register with registry
4. Import in shards as needed

### Error Handling

#### Session V2 Errors
- `InsufficientCapabilities` - Session lacks required capabilities
- `SessionAlreadyConsumed` - Trying to use consumed session
- `InvalidBundle` - Malformed bundle operations

#### General Errors
- Check instruction error codes
- Examine transaction logs
- Verify account states
- Validate PDA derivations