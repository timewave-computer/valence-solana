# API Reference

Complete reference for the Valence Protocol kernel program instructions, accounts, and data structures.

## Program Instructions

### Kernel Program Instructions

```mermaid
graph TD
    A[Kernel Program] --> B[initialize]
    A --> C[execute_capability]  
    A --> D[pause]
    A --> E[resume]
    
    B --> B1[Initialize system state]
    C --> C1[Execute capability with verification]
    D --> D1[Pause system operations]
    E --> E1[Resume system operations]
```

#### initialize
Initializes the kernel protocol system.

**Signature:**
```rust
pub fn initialize(ctx: Context<Initialize>) -> Result<()>
```

**Accounts:**
```rust
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + RegistryState::INIT_SPACE,
        seeds = [b"registry"],
        bump
    )]
    pub registry: Account<'info, RegistryState>,
    
    pub system_program: Program<'info, System>,
}
```

**Parameters:** None

**Returns:** `Result<()>`

#### execute_capability
Executes a capability with full verification chain.

**Signature:**
```rust
pub fn execute_capability(
    ctx: Context<ExecuteCapability>,
    capability_id: String,
    input_data: Vec<u8>
) -> Result<()>
```

**Accounts:**
```rust
#[derive(Accounts)]
pub struct ExecuteCapability<'info> {
    pub caller: Signer<'info>,
    
    #[account(mut)]
    pub session: Account<'info, SessionState>,
    
    #[account(
        mut,
        seeds = [b"registry"],
        bump
    )]
    pub registry: Account<'info, RegistryState>,
    
    pub system_program: Program<'info, System>,
}
```

**Parameters:**
- `capability_id: String` - Unique identifier for the capability to execute
- `input_data: Vec<u8>` - Serialized input data for the capability

**Returns:** `Result<()>`

#### pause
Pauses system operations.

**Signature:**
```rust
pub fn pause(ctx: Context<Pause>) -> Result<()>
```

**Accounts:**
```rust
#[derive(Accounts)]
pub struct Pause<'info> {
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"registry"],
        bump
    )]
    pub registry: Account<'info, RegistryState>,
}
```

**Parameters:** None

**Returns:** `Result<()>`

#### resume
Resumes system operations.

**Signature:**
```rust
pub fn resume(ctx: Context<Resume>) -> Result<()>
```

**Accounts:**
```rust
#[derive(Accounts)]
pub struct Resume<'info> {
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"registry"],
        bump
    )]
    pub registry: Account<'info, RegistryState>,
}
```

**Parameters:** None

**Returns:** `Result<()>`

## Account Structures

### RegistryState
Main system state account.

```rust
#[account]
pub struct RegistryState {
    /// Authority pubkey
    pub authority: Pubkey,
    
    /// Registry of available libraries
    pub libraries: Vec<LibraryEntry>,
    
    /// Registry of ZK programs
    pub zk_programs: Vec<ZkProgramEntry>,
    
    /// Dependencies between libraries
    pub dependencies: Vec<DependencyEntry>,
    
    /// System paused state
    pub paused: bool,
    
    /// Creation timestamp
    pub created_at: i64,
    
    /// Last update timestamp
    pub updated_at: i64,
}
```

### SessionState
Session configuration and state.

```rust
#[account]
pub struct SessionState {
    /// Session configuration
    pub config: SessionConfiguration,
    
    /// Current session status
    pub status: SessionStatus,
    
    /// Session permissions
    pub permissions: SessionPermissions,
    
    /// Session creation timestamp
    pub created_at: i64,
    
    /// Session expiration timestamp
    pub expires_at: i64,
    
    /// Usage statistics
    pub usage_stats: SessionUsageStats,
}
```

### LibraryEntry
Function library registry entry.

```rust
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LibraryEntry {
    /// Library name
    pub name: String,
    
    /// Library version
    pub version: String,
    
    /// Library program ID
    pub program_id: Pubkey,
    
    /// Library status
    pub status: LibraryStatus,
    
    /// Supported function types
    pub function_types: Vec<String>,
    
    /// Library dependencies
    pub dependencies: Vec<String>,
    
    /// Registration timestamp
    pub registered_at: i64,
}
```

## Data Structures

### Session Configuration

```rust
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SessionConfiguration {
    /// Maximum session duration in seconds
    pub max_duration: u64,
    
    /// Session permissions
    pub permissions: SessionPermissions,
    
    /// Session settings
    pub settings: SessionSettings,
}
```

### Session Permissions

```rust
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SessionPermissions {
    /// Can execute functions
    pub can_execute_functions: bool,
    
    /// Can compose functions
    pub can_compose_functions: bool,
    
    /// Can read cross-session data
    pub can_read_cross_session: bool,
    
    /// Can write cross-session data
    pub can_write_cross_session: bool,
}
```

### Capability Definition

```rust
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CapabilityDefinition {
    /// Capability identifier
    pub id: String,
    
    /// Capability type
    pub capability_type: CapabilityType,
    
    /// Capability scope
    pub scope: CapabilityScope,
    
    /// Required permissions
    pub required_permissions: Vec<String>,
    
    /// Associated functions
    pub functions: Vec<String>,
    
    /// Namespace restrictions
    pub namespace_restrictions: Vec<String>,
}
```

### Execution Context

```rust
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Capability being executed
    pub capability_id: String,
    
    /// Session identifier
    pub session_id: String,
    
    /// Session account (optional)
    pub session: Option<Pubkey>,
    
    /// Caller pubkey
    pub caller: Pubkey,
    
    /// Current block height
    pub block_height: u64,
    
    /// Current timestamp
    pub timestamp: i64,
    
    /// Remaining compute budget
    pub remaining_compute: u32,
    
    /// Input data for execution
    pub input_data: Vec<u8>,
}
```

## Function Types

### Verification Functions

```rust
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct VerificationInput {
    /// Verification context
    pub context: ExecutionContext,
    
    /// Verification parameters
    pub parameters: Vec<u8>,
    
    /// Additional verification data
    pub verification_data: Vec<u8>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct VerificationOutput {
    /// Verification result
    pub success: bool,
    
    /// Verification details
    pub details: String,
    
    /// Additional output data
    pub output_data: Vec<u8>,
}
```

### Function Composition

```rust
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FunctionChain {
    /// Chain identifier
    pub id: String,
    
    /// Function steps in order
    pub steps: Vec<FunctionStep>,
    
    /// Execution mode
    pub execution_mode: ExecutionMode,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FunctionStep {
    /// Function identifier
    pub function_id: String,
    
    /// Input mapping
    pub input_mapping: Vec<(String, String)>,
    
    /// Output mapping
    pub output_mapping: Vec<(String, String)>,
    
    /// Conditional execution
    pub condition: Option<String>,
}
```

## Error Types

### Registry Errors

```rust
#[error_code]
pub enum RegistryError {
    #[msg("Library not found")]
    LibraryNotFound,
    
    #[msg("Invalid library version")]
    InvalidVersion,
    
    #[msg("Dependency not satisfied")]
    DependencyNotSatisfied,
    
    #[msg("System is paused")]
    SystemPaused,
    
    #[msg("Unauthorized access")]
    Unauthorized,
}
```

### Verification Errors

```rust
#[error_code]
pub enum VerificationError {
    #[msg("Verification failed")]
    VerificationFailed,
    
    #[msg("Invalid verification input")]
    InvalidInput,
    
    #[msg("Verification timeout")]
    Timeout,
    
    #[msg("Unknown verification function")]
    UnknownFunction,
    
    #[msg("Permission denied")]
    PermissionDenied,
}
```

## Event Types

### Session Events

```rust
#[event]
pub struct SessionCreated {
    pub session_id: String,
    pub session_address: Pubkey,
    pub creator: Pubkey,
    pub configuration: SessionConfiguration,
}

#[event]
pub struct SessionActivated {
    pub session_id: String,
    pub session_address: Pubkey,
    pub activated_at: i64,
}

#[event]
pub struct SessionClosed {
    pub session_id: String,
    pub session_address: Pubkey,
    pub closed_at: i64,
    pub reason: String,
}
```

### Execution Events

```rust
#[event]
pub struct CapabilityExecutionStarted {
    pub capability_id: String,
    pub session_id: String,
    pub caller: Pubkey,
    pub started_at: i64,
}

#[event]
pub struct CapabilityExecutionCompleted {
    pub capability_id: String,
    pub session_id: String,
    pub caller: Pubkey,
    pub completed_at: i64,
    pub success: bool,
}

#[event]
pub struct VerificationCompleted {
    pub capability_id: String,
    pub verification_functions: Vec<String>,
    pub results: Vec<bool>,
    pub completed_at: i64,
}
```

## Usage Examples

### Initialize System

```rust
// Initialize the kernel protocol
let ctx = Context::new(
    program_id,
    &mut accounts,
    &[],
    BumpSeed::default(),
);

kernel::initialize(ctx)?;
```

### Execute Capability

```rust
// Execute a capability with verification
let capability_id = "example_capability".to_string();
let input_data = vec![1, 2, 3, 4]; // Serialized input

let ctx = Context::new(
    program_id,
    &mut accounts,
    &[],
    BumpSeed::default(),
);

kernel::execute_capability(ctx, capability_id, input_data)?;
```

### Build Execution Context

```rust
// Build execution context for capability
let context = ContextBuilder::new()
    .with_capability_id("example_capability".to_string())
    .with_session(session_pubkey)
    .with_caller(caller_pubkey)
    .with_input_data(input_data)
    .build()?;
```

## Query Patterns

### Session Queries

```rust
// Load session state
let session_account = &ctx.accounts.session;
let session_state = session_account.load()?;

// Check session permissions
if session_state.permissions.can_execute_functions {
    // Execute function
}
```

### Registry Queries

```rust
// Query available libraries
let registry = &ctx.accounts.registry;
let libraries = &registry.libraries;

// Find library by name
let library = libraries.iter()
    .find(|lib| lib.name == "example_library")
    .ok_or(RegistryError::LibraryNotFound)?;
```

## Singleton Programs

### Processor Singleton

Handles stateless execution orchestration.

**Instructions:**
- `initialize`: Initialize processor singleton
- `process_capability`: Process a capability execution
- `pause`: Pause processor operations
- `resume`: Resume processor operations

**State:**
```rust
pub struct ProcessorState {
    pub authority: Pubkey,
    pub is_paused: bool,
    pub total_processed: u64,
    pub bump: u8,
}
```

### Scheduler Singleton

Manages multi-shard scheduling and queue management.

**Instructions:**
- `initialize`: Initialize scheduler singleton
- `schedule_execution`: Schedule an operation
- `process_queue`: Process pending operations
- `register_shard`: Register shard with ordering rules
- `pause/resume`: Control scheduler operations

**State:**
```rust
pub struct SchedulerState {
    pub authority: Pubkey,
    pub is_paused: bool,
    pub max_shards: u16,
    pub max_queue_size: u16,
    pub active_shards: u16,
    pub queue_depth: u16,
    pub total_scheduled: u64,
    pub total_processed: u64,
    pub bump: u8,
}
```

### Diff Singleton

Calculates and optimizes state diffs.

**Instructions:**
- `initialize`: Initialize diff singleton
- `calculate_diff`: Calculate diff between states
- `process_diff_batch`: Process batch of diffs
- `optimize_batch`: Optimize diff batch
- `verify_diff_integrity`: Verify diff integrity
- `pause/resume`: Control diff operations

**State:**
```rust
pub struct DiffState {
    pub authority: Pubkey,
    pub is_paused: bool,
    pub max_batch_size: u16,
    pub optimization_level: OptimizationLevel,
    pub total_diffs_processed: u64,
    pub total_batches_optimized: u64,
    pub bump: u8,
}
``` 