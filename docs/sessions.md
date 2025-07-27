# Sessions

## The Challenge of Account-Based Programming

In account-based blockchains, accounts play a dual role that creates unique challenges for developers. They function both as ownership records—tracking who controls what assets—and as memory locations for program execution. This complicates application development, as programs must handle both aspects within the same abstraction.

Consider writing a program that interacts with an external lending protocol. When our program deposits tokens on behalf of a user, the lending protocol typically issues receipt tokens back to the depositor's address. This pattern, while standard in DeFi, prevents us from using a strictly linear type model where accounts are used only once. The external protocol forces account reuse, as the same account must be available to receive the voucher tokens.

Moreover, developers must manually manage state serialization, handle deterministic addressing, track account relationships, and ensure proper authorization. Low-level account management often overshadows the actual business logic.


```rust
/// Main session account that manages stateful operations
#[account]
pub struct Session {
    /// Determines authorization context
    pub scope: SessionScope,
    /// Controls access to this session
    pub guard: Guard,
    /// Owner of this session
    pub owner: Pubkey,
    /// Program that created this session
    pub program: Pubkey,
    /// For hierarchical authorization
    pub bound_to: Option<Pubkey>,
    /// For rate limiting
    pub usage_count: u64,
    /// Cross-session communication
    pub shared_data: SessionSharedData,
    /// program-specific metadata
    pub metadata: [u8; 64],
    /// Creation tracking
    pub created_at: i64,
    /// Execution tracking
    pub updated_at: i64,
}
```

## Sessions as Borrowed State

Sessions introduce a borrowing model inspired by Rust's ownership system. Rather than treating accounts as either immutable resources or mutable memory locations, sessions allow accounts to be borrowed for controlled periods of execution. This provides the flexibility to reuse accounts when necessary while maintaining memory safety guarantees.

When a session borrows an account, it establishes clear boundaries for access and modification. The session tracks operations, enforces authorization rules, and ensures proper release of borrowed accounts. Sessions can borrow multiple accounts, creating a cohesive execution context for complex DeFi operations.

## Transparent Account Abstraction

Sessions allow developers work with typed state objects and high-level operations, while allowing developers access to low-level state management when desired.

Account creation, state serialization, and address computation happen transparently. When sessions interact with other programs, they handle account passing and data marshaling automatically.

## Execution Context and Atomic Operations

Sessions provide a shared execution context enabling atomic operations across multiple accounts. This environment includes the timestamp, blockhash, caller identity, and protocol metadata.

Sessions track execution history with a use counter, allowing for rate limiting and sequential ordering. Shared data can be persisted across operations within the same session.

## Security Through Isolation

Each session represents a "guarded" security boundary. Guards are predicates that determine allowed operations based on state and execution context.

Sessions allow developers to enforce complex authorization patterns through guard composition. Hierarchical sessions enable delegation, where primary sessions grant limited authority to subsidiary sessions with their own security boundaries.

## Integration with Pure Functions

Sessions separate stateful coordination from pure business logic. While sessions handle account management and authorization, protocol logic is implemented as pure functions transforming state without side effects.

Sessions bridge the stateful blockchain environment and pure protocol logic by loading state, providing it to pure functions with execution context, and persisting transformed state back to accounts.

## Session Types

As the name implies, a "Session" implements a kind of session type, thus ensuring protocols follow intended communication patterns. Sessions enforce correct operational order and valid state transitions. This makes provides a natural way to express choreographies, for example: an escrow protocol might require deposits before withdrawals and restrict claims to designated recipients.