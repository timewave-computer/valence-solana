# Integration Patterns for Valence Protocol Singleton Modules

This document describes the integration patterns for working with Valence Protocol singleton modules within the unified kernel architecture. Since singleton modules are part of the kernel program, integration is through direct function calls rather than CPI.

## Processor Module Integration

The processor:: module handles stateless execution orchestration for capabilities.

### Direct Call Pattern Example

```rust
use anchor_lang::prelude::*;
use crate::processor::{ExecutionEngine, ContextBuilder, VerificationOrchestrator};

// Execute a capability via processor module
pub fn execute_via_processor(
    capability_definition: CapabilityDefinition,
    input_data: Vec<u8>,
    session: &AccountInfo,
    caller: Pubkey,
) -> Result<ExecutionResult> {
    // Build execution context
    let context = ContextBuilder::new()
        .with_capability_id(capability_definition.id.clone())
        .with_session(session.key())
        .with_caller(caller)
        .with_input_data(input_data)
        .build()?;
    
    // Orchestrate verification
    let verifier = VerificationOrchestrator::new();
    let verification_result = verifier.run_verification_chain(
        &capability_definition.verification_functions,
        &context
    )?;
    
    require!(verification_result.success, ProcessorError::VerificationFailed);
    
    // Execute capability
    let engine = ExecutionEngine::new();
    let result = engine.execute_capability(&capability_definition, &context)?;
    
    Ok(result)
}
```

### Key Points
- Processor maintains no state between executions
- All execution context must be built before calling
- Verification is orchestrated within the processor
- Gas limits and resource constraints are enforced

## Scheduler Module Integration

The scheduler:: module manages multi-shard scheduling and queue operations.

### Direct Call Pattern Example

```rust
use anchor_lang::prelude::*;
use crate::scheduler::{PartialOrderComposer, QueueManager, PriorityScheduler};
use crate::capabilities::{PartialOrder, OrderingConstraint};

// Schedule operations via scheduler module
pub fn schedule_operations(
    shards: Vec<(Pubkey, PartialOrder)>,
    scheduler_state: &mut Account<SchedulerState>,
) -> Result<Vec<Pubkey>> {
    // Compose partial orders from multiple shards
    let composer = PartialOrderComposer::new();
    let composed_order = composer.compose_orders(shards)?;
    
    // Create priority scheduler
    let mut scheduler = PriorityScheduler::new(scheduler_state);
    
    // Schedule operations based on composed order
    let execution_order = scheduler.schedule_with_constraints(
        composed_order.constraints,
        composed_order.priority
    )?;
    
    // Manage queue operations
    let queue_manager = QueueManager::new();
    queue_manager.enqueue_batch(execution_order.clone())?;
    
    Ok(execution_order)
}
```

### Queue Management
- Operations are processed based on priority
- Batch processing is available for efficiency
- Resource allocation is coordinated across shards
- Partial order constraints are respected through topological sorting

## Diff Module Integration

The diff:: module handles state diff calculations and optimizations.

### Direct Call Pattern Example

```rust
use anchor_lang::prelude::*;
use crate::diff::{DiffCalculator, BatchProcessor, Optimizer};

// Calculate and process diffs via diff module
pub fn process_state_diffs(
    old_state: Vec<u8>,
    new_state: Vec<u8>,
    diff_state: &mut Account<DiffState>,
) -> Result<DiffResult> {
    // Calculate diff
    let calculator = DiffCalculator::new();
    let diff_operations = calculator.calculate_diff(&old_state, &new_state)?;
    
    // Optimize diff batch
    let optimizer = Optimizer::new(diff_state.optimization_level);
    let optimized_diffs = optimizer.optimize_batch(diff_operations)?;
    
    // Process batch
    let batch_processor = BatchProcessor::new(diff_state.max_batch_size);
    let result = batch_processor.process_batch(optimized_diffs)?;
    
    // Update state metrics
    diff_state.total_diffs_processed += result.diffs_processed as u64;
    diff_state.total_batches_optimized += 1;
    
    Ok(result)
}
```

### Diff Operations
- Atomic diff processing ensures consistency
- Batch optimization reduces transaction costs
- Performance optimization algorithms are applied based on optimization level
- Diff validation prevents invalid state transitions

## Module State Initialization

Singleton module states are initialized as part of the kernel initialization or through dedicated module instructions.

### Initialization Pattern

```rust
// Initialize singleton module states during kernel initialization
pub fn initialize_kernel_with_modules(
    ctx: Context<Initialize>,
    processor_program: Pubkey,
) -> Result<()> {
    // Initialize shard state (includes eval config)
    let shard_state = &mut ctx.accounts.shard_state;
    shard_state.process_initialize(
        ctx.accounts.authority.key(),
        processor_program,
        ctx.bumps.shard_state
    )?;
    
    // Initialize scheduler state if needed
    if let Some(scheduler_state) = &mut ctx.accounts.scheduler_state {
        scheduler_state.initialize(
            ctx.accounts.authority.key(),
            16,  // max_shards
            1000, // max_queue_size
        )?;
    }
    
    // Initialize diff state if needed
    if let Some(diff_state) = &mut ctx.accounts.diff_state {
        diff_state.initialize(
            ctx.accounts.authority.key(),
            100, // max_batch_size
            OptimizationLevel::Advanced,
        )?;
    }
    
    Ok(())
}
```

## Best Practices

1. **Error Handling**: Always handle errors from module calls appropriately
2. **State Validation**: Verify module state before operations
3. **Resource Limits**: Be aware of compute unit limits when calling modules
4. **Batch Operations**: Use batch operations when possible for efficiency
5. **Monitoring**: Log module calls for debugging and monitoring
6. **Direct Calls**: Since modules are part of the kernel, use direct function calls for performance

## Common Patterns

### Sequential Module Calls
When multiple modules need to be called in sequence:

```rust
// First, schedule via scheduler module
let execution_order = schedule_operations(shards, scheduler_state)?;

// Then, execute via processor module
let execution_result = execute_via_processor(
    capability_definition,
    input_data,
    session,
    caller
)?;

// Finally, calculate diff
let diff_result = process_state_diffs(old_state, new_state, diff_state)?;
```

### Conditional Module Calls
Based on execution context:

```rust
if requires_scheduling {
    // Use scheduler for coordinated execution
    let queue_manager = QueueManager::new();
    queue_manager.enqueue_operation(operation)?;
} else {
    // Direct execution via processor
    let engine = ExecutionEngine::new();
    let result = engine.execute_capability(&capability_definition, &context)?;
}
```

### Shard-Based Execution Pattern
The most common pattern is executing capabilities through shards:

```rust
// Execute capability through shard with embedded eval
pub fn execute_capability(
    ctx: Context<ExecuteCapability>,
    capability_definition: CapabilityDefinition,
    input_data: Vec<u8>,
) -> Result<()> {
    let shard_state = &mut ctx.accounts.shard_state;
    
    // Shard validates and delegates to processor
    let result = shard_state.process_execute_capability(
        capability_definition.id,
        input_data,
        &execution_context
    )?;
    
    // Handle result
    match result {
        ExecutionResult { success: true, .. } => {
            msg!("Capability executed successfully");
            Ok(())
        },
        ExecutionResult { success: false, .. } => {
            Err(ValenceError::ExecutionFailed.into())
        }
    }
}
```
