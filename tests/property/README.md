# Valence Protocol Property Tests

This directory contains property-based tests that verify critical security invariants of the Valence Protocol.

## Test Modules

### 1. Session Security (`session_security.rs`)
Tests properties related to session management:
- Session IDs are unique
- State transitions follow valid patterns
- Capabilities cannot be escalated
- Session expiry is monotonic
- Concurrent access maintains safety
- Delegation preserves security invariants

### 2. Capability Enforcement (`capability_enforcement.rs`)
Verifies capability-based access control:
- Capability checks are consistent and deterministic
- Inheritance maintains subset relationships
- Admin implications are properly enforced
- Revocation is complete and immediate
- Delegation depth limits are respected
- Contextual adjustments preserve security

### 3. Function Registration (`function_registration.rs`)
Ensures function registry integrity:
- Function IDs are deterministic
- Registration is idempotent
- Version history is maintained
- Capability requirements are enforced
- Deregistration is atomic
- Bytecode validation works correctly
- Metadata constraints are respected

### 4. State Isolation (`state_isolation.rs`)
Verifies isolation between components:
- Function states are isolated
- Session states don't interfere
- Account data is separate
- Concurrent access maintains isolation
- Memory regions don't overlap
- Cross-function calls preserve boundaries

### 5. Resource Limits (`resource_limits.rs`)
Tests resource consumption limits:
- Compute unit limits are enforced
- Memory allocation respects limits
- Account sizes are bounded
- Transaction size limits work
- Rate limiting functions correctly
- Call depth is limited
- Session concurrency is controlled
- Storage rent is properly calculated

### 6. Authorization (`authorization.rs`)
Verifies authentication and authorization:
- Authorization is transitive where appropriate
- Signature validation is deterministic
- Multi-signature thresholds are enforced
- Permission delegation preserves hierarchy
- Time-based authorization expires correctly
- Revocation is immediate and complete

## Running the Tests

Run all property tests:
```bash
cargo test -p valence-property-tests
```

Run a specific test module:
```bash
cargo test -p valence-property-tests session_security
```

Run with more test cases (slower but more thorough):
```bash
PROPTEST_CASES=10000 cargo test -p valence-property-tests
```

Run with a specific seed for reproducibility:
```bash
PROPTEST_RNG_SEED=12345 cargo test -p valence-property-tests
```

## Configuration

Property tests can be configured through environment variables:
- `PROPTEST_CASES`: Number of test cases to generate (default: 256)
- `PROPTEST_MAX_SHRINK_ITERS`: Maximum shrinking iterations (default: 10000)
- `PROPTEST_RNG_SEED`: Random seed for reproducibility

## Security Properties

The property tests verify the following high-level security properties:

1. **Isolation**: Components cannot interfere with each other
2. **Determinism**: Same inputs always produce same outputs
3. **Monotonicity**: Certain values only move in one direction
4. **Transitivity**: Relationships follow expected patterns
5. **Atomicity**: Operations either fully succeed or fully fail
6. **Bounded Resources**: All resources have enforced limits
7. **Authorization**: Access control is properly enforced

## Debugging Failed Tests

When a property test fails:

1. The test framework will try to shrink the input to find a minimal failing case
2. The shrunken input will be printed with the failure
3. You can reproduce the failure using the printed seed
4. Add the minimal case as a regression test

## Best Practices

1. Keep properties simple and focused
2. Use descriptive names for properties
3. Generate realistic input distributions
4. Consider edge cases in generators
5. Document what each property verifies
6. Add regression tests for found bugs