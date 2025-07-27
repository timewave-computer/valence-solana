# Guards

## The Authorization Challenge

Secure authorization must be auditable i.e. minimal. Yet integrating with DeFi protocols requires expressive authorization rules. Blockchain constraints make permission type hierarchies inefficient.

Therefore, achieving both an expressive and performant authorization system, requires a pluggable authorization interface.

## Guards as Pure Predicates

Guards treat authorization as pure functions that examine state, operation, and context to return a boolean decision. This functional model makes guards predictable, testable, and composable without side effects or state mutations.

By expressing authorization as predicates rather than procedures, guards enable powerful optimizations. State-independent guards can be cached, multiple conditions can be evaluated in parallel, and entire authorization policies become understandable by examining guard configuration rather than tracing imperative code.

## Composable Authorization

Guards compose through logical operators—AND requires multiple conditions, OR allows alternatives, NOT inverts conditions.

Common patterns like time windows, rate limiting, or ownership checks can be defined once and combined differently for different operations. This reuse ensures consistent security policies while reducing code duplication.

## Built-in and External Guards

The system provides built-in guards for common patterns: OwnerOnly for ownership checks, Expiration for time-based control, UsageLimit for rate limiting, and Permission for authorization levels. These battle-tested implementations handle most authorization needs efficiently.

```rust
/// Unified guard system for all authorization logic
#[derive(Clone, Debug)]
pub enum Guard {
    // Simple Guards
    AlwaysTrue,
    AlwaysFalse,
    OwnerOnly,
    Expiration { expires_at: i64 },
    Permission { required: u64 },
    UsageLimit { max: u64 },
    
    // Composite Guards
    And(Box<Guard>, Box<Guard>),
    Or(Box<Guard>, Box<Guard>),
    Not(Box<Guard>),
    
    // External Guards
    External { program: Pubkey, data: Vec<u8> },
}
```

For domain-specific logic, external guards enable custom predicates as separate programs. They maintain the pure predicate model—receiving state information but unable to modify it. The standardized interface allows external guards to be shared and reused across applications.

```rust
/// Input data for external guard programs
pub struct ExternalGuardInput {
    pub guard_data: Vec<u8>,
    pub operation: Vec<u8>,
    pub timestamp: i64,
    pub slot: u64,
    pub caller: Pubkey,
    pub recent_blockhash: [u8; 32],
}

/// Output from external guard evaluation
pub struct ExternalGuardOutput {
    pub authorized: bool,
    pub reason: Option<String>,
}
```

## Guard Security

Guard security comes from constraints. Pure predicates prevent state modification vulnerabilities, race conditions, and reentrancy. Compositional policies make authorization explicit and auditable—the entire policy is visible as a guard tree rather than hidden in procedural code.

Resource limits provide additional security. Guards must evaluate within computational bounds, preventing denial-of-service through complex logic. The system limits composition depth and external guard data size, ensuring efficient authorization checks.

## Integration with Sessions

Guards integrate with sessions to provide comprehensive access control. Each session's guard defines its authorization policy, creating a security boundary for all accessible state. This makes authorization a first-class concern—developers must consider access control when creating sessions, not as an afterthought.

The combination enables sophisticated delegation. Primary sessions might have broad permissions while child sessions have restrictive guards, naturally implementing principle of least privilege.

## Conclusion

Guards transform authorization from scattered checks into declarative, composable policies. The pure predicate model makes authorization predictable and auditable. Built-in guards handle common patterns while external guards enable custom logic without sacrificing security benefits.

When combined with sessions, guards provide a complete authorization framework that scales from simple ownership checks to complex multi-party protocols, making correct authorization as natural as writing business logic.