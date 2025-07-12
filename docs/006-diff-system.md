# Diff-Based State Management

## Overview

Valence implements a diff-based state management system that tracks all state transitions through a cryptographic hash chain. Each operation produces a deterministic state hash based on the previous state and the operation's effects, creating an immutable audit trail of all changes.

## Core Concepts

### State Hash

Every session maintains a `state_hash` - a 32-byte value that represents the current state of execution. This hash is computed deterministically from:
- The previous state hash
- The operation's result data
- The operation's parameters

```rust
pub struct Session {
    pub state_hash: [u8; 32],
    // ... other fields
}
```

### Hash Chain

State transitions form a hash chain where each new state depends on the previous:

```
Initial State (H₀) → Operation₁ → State (H₁) → Operation₂ → State (H₂) → ...
```

This creates several important properties:
- **Deterministic**: Same operations in same order always produce same final hash
- **Tamper-evident**: Cannot modify history without changing all subsequent hashes
- **Verifiable**: Can replay operations to verify final state

## Implementation

### Computing State Transitions

The current implementation uses a simple hash chain mechanism:

```rust
fn compute_next_hash(prev_hash: [u8; 32], operation_data: &[u8]) -> [u8; 32] {
    // Simple hash chain - in production use proper hashing
    let mut next_hash = prev_hash;
    if !operation_data.is_empty() {
        next_hash[0] = next_hash[0].wrapping_add(operation_data[0]);
        next_hash[31] = next_hash[31].wrapping_add(operation_data[operation_data.len() - 1]);
    }
    next_hash
}
```

**Note**: The current implementation uses a simplified hash function for development. Production systems should use cryptographic hash functions like SHA-256 or Blake3.

### Operation Structure

Each operation in a bundle can specify an expected diff hash:

```rust
pub struct Operation {
    /// Function hash (lookup in registry)
    pub function_hash: [u8; 32],
    /// Arguments for the function
    pub args: Vec<u8>,
    /// Expected diff hash after execution
    pub expected_diff: Option<[u8; 32]>,
}
```

If `expected_diff` is provided, the system verifies that the computed state hash matches the expected value after executing the operation.

## Execution Modes

### Synchronous Execution

In sync mode, all operations execute atomically within a single transaction:

```rust
// Execute bundle synchronously
let bundle = Bundle {
    operations: vec![op1, op2, op3],
    mode: ExecutionMode::Sync,
};

// All operations execute in order
// Final state hash is computed and stored
```

Benefits:
- Atomic execution - all or nothing
- Lower latency
- Simpler error handling

Limitations:
- Constrained by transaction size limits
- All operations must fit in one transaction

### Asynchronous Execution

In async mode, operations can span multiple transactions with checkpoints:

```rust
pub struct ExecutionState {
    /// Current operation index
    pub current_operation: u16,
    /// Total operations
    pub total_operations: u16,
    /// Current state hash
    pub state_hash: [u8; 32],
    /// Whether execution is complete
    pub is_complete: bool,
    /// The bundle operations (stored for continuation)
    pub operations: Vec<Operation>,
}
```

Benefits:
- Can handle large bundles
- Resume from checkpoints after failures
- Better resource utilization

Process:
1. Start async bundle → creates ExecutionState
2. Execute operations in batches
3. Update state hash and checkpoint after each batch
4. Continue until all operations complete

## Use Cases

### 1. Multi-Step Workflows

Complex operations that require multiple steps:
```rust
let bundle = Bundle {
    operations: vec![
        Operation { function_hash: hash("validate_inputs"), ... },
        Operation { function_hash: hash("process_data"), ... },
        Operation { function_hash: hash("update_state"), ... },
        Operation { function_hash: hash("emit_events"), ... },
    ],
    mode: ExecutionMode::Async,
};
```

### 2. Optimistic Execution

Clients can pre-compute expected state hashes:
```rust
let op = Operation {
    function_hash: hash("transfer"),
    args: serialize(&TransferArgs { amount: 100, to: recipient }),
    expected_diff: Some(precomputed_hash), // Verify execution matches expectation
};
```

### 3. Execution Verification

Verify that a sequence of operations produces expected state:
```rust
// Start with initial state
let mut state = initial_hash;

// Replay operations
for op in operations {
    let result = execute_operation(op);
    state = compute_next_hash(state, &result);
    
    // Verify if expected
    if let Some(expected) = op.expected_diff {
        assert_eq!(state, expected);
    }
}

// Final state should match session state
assert_eq!(state, session.state_hash);
```

## Best Practices

### 1. Deterministic Operations

Ensure functions produce deterministic results:
- Avoid using timestamps in computations
- Sort unordered data before hashing
- Use deterministic random sources if needed

### 2. Efficient Checkpointing

For async execution:
- Batch operations logically (e.g., validation → processing → finalization)
- Set checkpoints at natural boundaries
- Consider gas costs when sizing batches

### 3. Hash Verification

When using expected diffs:
- Pre-compute hashes during bundle construction
- Use for critical operations where outcome must be certain
- Handle mismatch errors gracefully

## Security Considerations

### Hash Function Selection

The production implementation should use a cryptographic hash function that provides:
- Collision resistance
- Pre-image resistance  
- Second pre-image resistance

Recommended: SHA-256, SHA-3, or Blake3

### State Recovery

The diff system enables state recovery by replaying operations:
1. Start from a known good state
2. Replay operations in order
3. Verify intermediate hashes if available
4. Recover to any point in the execution history

### Audit Trail

The hash chain provides a complete audit trail:
- Every state transition is recorded
- Cannot modify history without detection
- Can prove execution path cryptographically

## Future Enhancements

### Merkle Tree Integration

Instead of a simple hash chain, use Merkle trees for:
- Efficient proof generation
- Partial state verification
- Parallel operation execution

### Zero-Knowledge Proofs

Generate ZK proofs of state transitions:
- Prove execution without revealing operations
- Verify complex workflows efficiently
- Enable privacy-preserving execution

### State Compression

Implement state compression techniques:
- Store only hash differences
- Compress operation sequences
- Reduce on-chain storage costs

## Example Implementation

Here's a complete example of using the diff system:

```rust
// Create a session with initial state
let session = create_session(
    capabilities: vec!["read", "write"],
    init_state_hash: [0u8; 32], // Starting state
);

// Build a bundle with expected outcomes
let bundle = Bundle {
    operations: vec![
        Operation {
            function_hash: hash("read_balance"),
            args: account_bytes,
            expected_diff: None, // Don't verify this step
        },
        Operation {
            function_hash: hash("update_balance"),
            args: new_balance_bytes,
            expected_diff: Some(expected_hash), // Verify this produces expected state
        },
    ],
    mode: ExecutionMode::Sync,
};

// Execute bundle
execute_sync_bundle(session, bundle)?;

// Session now has new state_hash reflecting all operations
assert_eq!(session.state_hash, expected_final_state);
```

The diff system provides the foundation for verifiable, auditable state management in Valence, enabling complex multi-step operations while maintaining cryptographic guarantees about execution integrity.