# ZK Transfer Limit Example

This example demonstrates how to use Valence's architecture to implement privacy-preserving transfer limits using zero-knowledge proofs and move semantics.

## Overview

The example shows:

- **ZK Verification Gateway Integration**: How to use a committee-managed ZK verifier as a registered function
- **Move Semantics**: Clean ownership transfer without shared mutable state

## Architecture

## Key Concepts

### 1. ZK Verification as a Registered Function

Instead of building ZK verification into the kernel, it's implemented as a registered function, which allows for managed upgrades:

```rust
// Register the ZK verifier
let zk_verifier = RegisteredProgram {
    address: verifier_program_address,
    active: true,
    label: *b"zk_verifier_gateway_____________",
};
```

### 2. Move Semantics for Position Transfer

The example uses Valence SDK's move semantics patterns:

```rust
// Implement move semantics for the position
implement_move_semantics!(TransferLimitPosition);

// Transfer ownership cleanly
position.transfer_ownership(new_owner)?;

// Old owner can no longer access
session.invalidate()?;
```

### 3. Privacy-Preserving Transfer Limits

- Daily limits are hidden using commitments
- Users prove transfers are within limits without revealing the limit
- Only the amount transferred today is public

## Status

**Note**: This example is currently undergoing a complete rewrite to align with the new valence-kernel architecture. The example code has been temporarily disabled while the implementation is updated to work with the simplified kernel design.

## Code Structure

- `main.rs` - Placeholder indicating example needs rewrite for new architecture

## Production Considerations

In a production deployment:

1. **ZK Verifier**: Would be a properly audited program managed by a security committee
2. **Proof Generation**: Would use actual ZK circuits with proper constraints
3. **Commitments**: Would use secure commitment schemes (Pedersen, etc.)
4. **Move Semantics**: Would include proper error handling and state cleanup

## Key Takeaways

1. **Flexibility**: Valence doesn't prescribe a specific ZK system - protocols choose their own
2. **Composability**: ZK verification is just another registered function
3. **Clean Ownership**: Move semantics prevent concurrent access issues
4. **Privacy**: Complex privacy requirements can be implemented at the protocol level

## Related Examples

- See `examples/direct_operations.rs` for basic kernel operations
- See `examples/batch_operations.rs` for batch execution examples
- See `e2e/` for comprehensive integration tests