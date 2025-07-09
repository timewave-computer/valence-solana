# CPI Integration Patterns for Valence Protocol Singletons

This document describes the Cross-Program Invocation (CPI) patterns for integrating with Valence Protocol singleton programs.

## Processor Singleton Integration

The processor singleton handles stateless execution orchestration for capabilities.

### CPI Pattern Example

```rust
use anchor_lang::prelude::*;
use processor::{cpi, ProcessCapability};

// Execute a capability via processor singleton
pub fn execute_via_processor(
    ctx: Context<ExecuteViaProcessor>,
    capability_id: String,
    input_data: Vec<u8>,
) -> Result<()> {
    let cpi_accounts = cpi::ProcessCapability {
        processor_state: ctx.accounts.processor_state.to_account_info(),
        caller: ctx.accounts.caller.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new(
        ctx.accounts.processor_program.to_account_info(),
        cpi_accounts,
    );
    
    cpi::process_capability(cpi_ctx, capability_id, input_data)?;
    
    Ok(())
}
```

### Key Points
- Processor maintains no state between executions
- All execution context must be provided in the CPI call
- Verification functions are orchestrated by the processor
- Gas limits and resource constraints are enforced

## Scheduler Singleton Integration

The scheduler singleton manages multi-shard scheduling and queue operations.

### CPI Pattern Example

```rust
use anchor_lang::prelude::*;
use scheduler::{cpi, EnqueueOperation, OperationType};

// Queue an operation via scheduler singleton
pub fn queue_via_scheduler(
    ctx: Context<QueueViaScheduler>,
    operation: PendingOperation,
) -> Result<()> {
    let cpi_accounts = cpi::EnqueueOperation {
        scheduler_state: ctx.accounts.scheduler_state.to_account_info(),
        queue: ctx.accounts.session_queue.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new(
        ctx.accounts.scheduler_program.to_account_info(),
        cpi_accounts,
    );
    
    cpi::enqueue_operation(cpi_ctx, operation)?;
    
    Ok(())
}
```

### Queue Management
- Operations are processed based on priority
- Batch processing is available for efficiency
- Resource allocation is coordinated across shards
- Partial order constraints are respected

## Diff Singleton Integration

The diff singleton handles state diff calculations and optimizations.

### CPI Pattern Example

```rust
use anchor_lang::prelude::*;
use diff::{cpi, CalculateDiff};

// Calculate diff via diff singleton
pub fn calculate_diff_via_cpi(
    ctx: Context<CalculateDiffViaCPI>,
    old_state: Vec<u8>,
    new_state: Vec<u8>,
) -> Result<()> {
    let cpi_accounts = cpi::CalculateDiff {
        diff_state: ctx.accounts.diff_state.to_account_info(),
        caller: ctx.accounts.caller.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new(
        ctx.accounts.diff_program.to_account_info(),
        cpi_accounts,
    );
    
    cpi::calculate_diff(cpi_ctx, old_state, new_state)?;
    
    Ok(())
}
```

### Diff Operations
- Atomic diff processing ensures consistency
- Batch optimization reduces transaction costs
- Performance optimization algorithms are applied automatically
- Diff validation prevents invalid state transitions

## Singleton Initialization

All singleton programs must be initialized before use.

### Initialization Pattern

```rust
// Initialize processor singleton
pub fn initialize_processor(ctx: Context<InitializeProcessor>) -> Result<()> {
    let cpi_accounts = processor::cpi::Initialize {
        processor_state: ctx.accounts.processor_state.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new(
        ctx.accounts.processor_program.to_account_info(),
        cpi_accounts,
    );
    
    processor::cpi::initialize(cpi_ctx)?;
    Ok(())
}
```

## Best Practices

1. **Error Handling**: Always handle CPI errors appropriately
2. **Account Validation**: Verify account ownership before CPI calls
3. **Resource Limits**: Be aware of compute unit limits when making CPI calls
4. **Batch Operations**: Use batch operations when possible for efficiency
5. **Monitoring**: Log CPI calls for debugging and monitoring

## Common Patterns

### Sequential CPI Calls
When multiple singletons need to be called in sequence:

```rust
// First, process via scheduler
scheduler::cpi::schedule_execution(scheduler_ctx, shard_id, capabilities)?;

// Then, execute via processor
processor::cpi::process_capability(processor_ctx, capability_id, input)?;

// Finally, calculate diff
diff::cpi::calculate_diff(diff_ctx, old_state, new_state)?;
```

### Conditional CPI Calls
Based on execution context:

```rust
if requires_scheduling {
    scheduler::cpi::enqueue_operation(scheduler_ctx, operation)?;
} else {
    // Direct execution
    processor::cpi::process_capability(processor_ctx, capability_id, input)?;
}
```

## Migration Guide

For programs migrating from the old architecture:

1. Replace direct eval calls with processor CPI
2. Replace queue management with scheduler CPI
3. Replace optimization calls with diff CPI
4. Update account contexts to include singleton program accounts
5. Test CPI integration thoroughly

## Troubleshooting

Common issues and solutions:

- **Account not found**: Ensure singleton is initialized
- **Insufficient compute units**: Increase compute budget
- **CPI depth exceeded**: Reduce nested CPI calls
- **Program not executable**: Verify program deployment