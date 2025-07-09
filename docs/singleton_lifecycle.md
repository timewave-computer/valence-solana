# Singleton Program Lifecycle Management

This document describes the initialization and lifecycle management of Valence Protocol singleton programs.

## Overview

The Valence Protocol uses three singleton programs that must be properly initialized and managed:

1. **Processor Singleton**: Stateless execution orchestration
2. **Scheduler Singleton**: Multi-shard scheduling and queue management
3. **Diff Singleton**: State diff calculation and optimization

## Initialization Sequence

Singletons should be initialized in the following order:

### 1. Processor Singleton

The processor must be initialized first as other components may depend on it.

```rust
// Initialize processor with authority
processor::initialize(
    ctx,
    ProcessorConfig {
        max_verification_functions: 10,
        max_compute_units: 1_000_000,
        pause_authority: authority.key(),
    }
)?;
```

### 2. Scheduler Singleton

Initialize after processor to enable scheduling capabilities.

```rust
// Initialize scheduler
scheduler::initialize(
    ctx,
    SchedulerConfig {
        max_shards: 100,
        max_queue_size: 1000,
        default_priority: 5,
    }
)?;
```

### 3. Diff Singleton

Initialize last as it may use both processor and scheduler.

```rust
// Initialize diff
diff::initialize(
    ctx,
    DiffConfig {
        max_batch_size: 50,
        optimization_level: OptimizationLevel::Aggressive,
    }
)?;
```

## State Management

### Processor State
- No persistent state between executions
- Configuration stored on-chain
- Pause/resume functionality for maintenance

### Scheduler State
- Active shard registry
- Queue state and statistics
- Resource allocation metrics
- Partial order constraint cache

### Diff State
- Diff operation history (optional)
- Performance metrics
- Optimization statistics

## Lifecycle Operations

### Pause Operations

All singletons support pause functionality for maintenance:

```rust
// Pause processor
processor::pause(ctx)?;

// Pause scheduler (queues operations)
scheduler::pause(ctx)?;

// Pause diff
diff::pause(ctx)?;
```

### Resume Operations

Resume normal operations after maintenance:

```rust
// Resume in reverse order of pause
diff::resume(ctx)?;
scheduler::resume(ctx)?;
processor::resume(ctx)?;
```

### Configuration Updates

Update singleton configurations without redeployment:

```rust
// Update processor config
processor::update_config(
    ctx,
    ProcessorConfigUpdate {
        max_compute_units: Some(2_000_000),
        ..Default::default()
    }
)?;
```

## Health Monitoring

### Processor Health
- Monitor verification success rate
- Track average execution time
- Check compute unit usage

### Scheduler Health
- Queue depth and processing rate
- Shard coordination efficiency
- Constraint resolution metrics

### Diff Health
- Diff calculation performance
- Optimization effectiveness
- Batch processing throughput

## Error Recovery

### Processor Errors
- Automatic retry for transient failures
- Fallback to direct execution if processor unavailable
- Error logging for debugging

### Scheduler Errors
- Dead letter queue for failed operations
- Manual intervention for stuck queues
- Priority adjustment for recovery

### Diff Errors
- Validation before diff application
- Rollback capability for failed diffs
- Batch splitting for large operations

## Upgrade Procedures

### Rolling Updates
1. Deploy new version to devnet
2. Test integration with existing programs
3. Pause production singleton
4. Deploy upgrade
5. Run migration if needed
6. Resume operations
7. Monitor for issues

### Emergency Procedures
- Emergency pause authority
- Rollback procedures
- Data recovery protocols

## Best Practices

1. **Regular Health Checks**: Monitor singleton health metrics
2. **Capacity Planning**: Plan for growth in usage
3. **Error Handling**: Implement proper error handling for CPI calls
4. **Testing**: Test singleton integration in all environments
5. **Documentation**: Keep initialization parameters documented
6. **Monitoring**: Set up alerts for singleton issues
7. **Backup Plans**: Have fallback strategies for singleton failures

## Common Issues

### Initialization Failures
- Check authority permissions
- Verify account sizes
- Ensure unique PDAs

### CPI Failures
- Check program deployment
- Verify account ownership
- Monitor compute limits

### Performance Issues
- Review batch sizes
- Check optimization settings
- Monitor resource usage