# Integration Guide

This guide walks through building applications on Valence, from basic shard development to advanced integration patterns. It assumes familiarity with Solana development and the Valence architecture.

## Shard Development

Shards are user-deployed programs that compose Valence kernel services. They manage application state and orchestrate function execution within capability boundaries.

### Basic Shard Structure

Every shard needs several core components:

```rust
#[program]
pub mod my_shard {
    use super::*;
    
    pub fn initialize(ctx: Context<Initialize>, config: Config) -> Result<()> {
        // Set up shard configuration
    }
    
    pub fn execute_action(ctx: Context<ExecuteAction>, data: Vec<u8>) -> Result<()> {
        // Application logic using imported functions
    }
}
```

The shard acts as a coordinator, importing functions from the registry and managing sessions for secure execution.

### Configuration Management

Shard configuration should support both flexibility and safety:

- Store authority for administrative operations
- Set limits on bundle sizes to prevent DoS
- Configure default policies for function imports
- Include pause mechanisms for emergencies

Configuration updates should be rare and require strong authorization. Most policy decisions should happen at the session level rather than globally.

### Session Integration

Sessions provide the security context for operations. Design your session strategy around your application's access patterns:

For financial applications, create sessions per user with specific asset permissions. For gaming applications, create sessions per match with game-specific capabilities. For governance applications, create sessions per proposal with voting capabilities.

The key is mapping your application's permission model to Valence's capability system.

### Function Import Strategy

Functions are the building blocks of shard logic. Develop a clear strategy for managing imports:

**Static Imports**: Import all required functions during shard initialization. This approach provides predictability but requires redeployment for updates.

**Dynamic Imports**: Import functions on-demand as features are activated. This enables flexibility but complicates permission management.

**Hybrid Approach**: Import core functions statically and auxiliary functions dynamically. This balances stability with extensibility.

Consider deregistration policies carefully. Respecting deregistration allows function updates but risks breaking changes. Ignoring deregistration ensures stability but prevents security updates.

## Function Implementation

Functions are stateless programs that perform specific operations. They receive input data and return results without maintaining persistent state.

### Function Design Principles

Functions should be:
- **Single-purpose**: Do one thing well
- **Deterministic**: Same input produces same output
- **Side-effect free**: No hidden state changes
- **Composable**: Work well with other functions

Avoid creating monolithic functions that try to handle multiple concerns. Instead, create focused functions that shards can compose.

### Input/Output Conventions

Establish clear conventions for function interfaces:

```rust
pub fn process(ctx: Context<Process>, input: InputData) -> Result<OutputData> {
    // Validate input
    // Perform operation
    // Return result
}
```

Use strongly-typed structures for input and output rather than raw bytes. This improves safety and makes functions self-documenting.

### Error Handling

Functions should fail fast with clear errors:

- Validate all inputs immediately
- Return specific error variants
- Include relevant context in errors
- Never panic in production code

Well-designed errors make debugging easier and improve the developer experience.

### Testing Functions

Test functions in isolation before integration:

1. Unit test core logic
2. Test edge cases and error conditions
3. Verify deterministic behavior
4. Check resource consumption

Functions execute in constrained environments, so profile resource usage during testing.

## Verifier Development

Verifiers implement pluggable verification logic for security policies. They evaluate predicates and return boolean results.

### Verifier Interface

Verifiers implement a simple interface:

```rust
pub fn verify(
    ctx: Context<Verify>,
    predicate: PredicateData,
    context: ContextData,
) -> Result<bool> {
    // Evaluate predicate against context
    // Return true if valid, false otherwise
}
```

The predicate contains the condition to check, while the context provides the data to check against.

### Common Verification Patterns

**Balance Verification**: Check if an account holds sufficient tokens
**Signature Verification**: Validate cryptographic signatures
**Time-based Verification**: Ensure operations happen within time windows
**Merkle Proof Verification**: Validate inclusion in Merkle trees
**Threshold Verification**: Check if enough approvals exist

Design verifiers to be reusable across different applications.

### Security Considerations

Verifiers are security-critical components:

- Never trust unvalidated input
- Prevent resource exhaustion attacks
- Avoid timing attacks in cryptographic operations
- Return minimal information in failure cases
- Log security-relevant events

Since verifiers gate access to sensitive operations, they must be thoroughly audited.

## Off-Chain Services

Off-chain services complement on-chain logic by handling complex computations and external interactions.

### Session Builder Pattern

The session builder demonstrates a common service pattern:

1. Monitor on-chain events
2. Process requests off-chain
3. Submit results on-chain
4. Handle failures gracefully

This pattern works for various services like oracles, keepers, and automation.

### Service Architecture

Design services for reliability:

- Use persistent storage for state
- Implement proper retry logic
- Handle concurrent requests
- Monitor service health
- Provide operational metrics

Services should be stateless where possible, deriving all necessary information from on-chain data.

### Integration Points

Services typically integrate at specific points:

**Session Initialization**: Build complex initial states
**Bundle Construction**: Optimize operation ordering
**State Verification**: Validate execution results
**Event Processing**: React to on-chain events
**External Data**: Bring off-chain data on-chain

Identify which integration points your application needs and design services accordingly.

## Testing Strategies

Comprehensive testing ensures reliable applications:

### Unit Testing

Test individual components in isolation:
- Shard instructions
- Function logic
- Verifier predicates
- Service components

Use Solana's testing frameworks to simulate on-chain behavior.

### Integration Testing

Test component interactions:
- Function imports and execution
- Session lifecycle flows
- Bundle execution patterns
- Service coordination

Create test scenarios that exercise full workflows.

### Security Testing

Focus on security-critical paths:
- Capability enforcement
- Authority validation
- State consistency
- Error handling

Attempt to break security assumptions and verify defenses hold.

### Performance Testing

Measure resource consumption:
- Transaction size limits
- Computation budgets
- Account rent costs
- Service throughput

Optimize hot paths while maintaining security.

## Deployment Workflows

Deploying Valence applications requires coordination:

### Development Deployment

1. Deploy functions to devnet
2. Register with devnet registry
3. Deploy shard program
4. Initialize shard configuration
5. Import required functions
6. Deploy off-chain services
7. Test full integration

Use devnet liberally during development.

### Production Deployment

1. Security audit all components
2. Deploy functions to mainnet
3. Register with mainnet registry
4. Deploy shard with conservative limits
5. Import functions with careful policies
6. Deploy services with monitoring
7. Gradual rollout with limits

Production deployments should be cautious and reversible.

### Upgrade Strategies

Plan for upgrades from the start:

- Use versioned function hashes
- Implement deprecation warnings
- Support multiple function versions
- Provide migration tools
- Document breaking changes

Smooth upgrades maintain user confidence.

## Migration from Traditional Programs

Migrating existing Solana programs to Valence requires rethinking architecture:

### Decomposition Strategy

1. Identify core functionality
2. Extract stateless operations as functions
3. Design capability model
4. Implement session management
5. Add verification logic
6. Create migration path

Start with non-critical features to gain experience.

### Incremental Migration

Migrate incrementally rather than all at once:

- Run old and new systems in parallel
- Migrate users gradually
- Maintain compatibility layers
- Monitor for issues
- Document differences

This reduces risk and provides rollback options.

### Common Challenges

Address these common migration challenges:

**State Management**: Move from ambient state to capability-gated state
**Permission Models**: Map existing permissions to capabilities
**Atomicity**: Redesign for bundle-based execution
**Upgradability**: Plan for function evolution

Each challenge has solutions within Valence's model.

## Best Practices

Follow these practices for successful Valence applications:

### Design Practices

- Start with clear capability models
- Design for composability
- Plan for evolution
- Document extensively
- Consider failure modes

### Implementation Practices

- Validate all inputs
- Handle errors gracefully
- Log important events
- Monitor system health
- Test comprehensively

### Operational Practices

- Deploy incrementally
- Monitor continuously
- Respond quickly to issues
- Communicate changes clearly
- Maintain compatibility

Success requires attention at all levels, from design through operations.