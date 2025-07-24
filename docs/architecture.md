# Valence Architecture

## Overview

Valence is a minimal secure microkernel for Solana programs that provides fundamental mechanisms for authorization and execution. The core follows the principle that a microkernel should provide mechanisms, not policies.

## Core Design Principles

### 1. Mechanisms, Not Policies

The microkernel provides:
- Session and account management with verifier-based authorization
- Verification delegation with CPI depth optimization
- Usage tracking and metadata storage
- Hierarchical composition

Users implement:
- Authorization logic (policy via verifiers)
- Business logic (policy via programs)
- State management (policy via protocols)

### 2. Zero-Copy by Default

All core state uses fixed-size fields:
- No `Vec<T>` in account state (except session accounts list)
- No `String` types
- Predictable memory layout
- Efficient deserialization

### 3. Composition Over Configuration

Instead of complex configuration:
- Deploy custom verifier for authorization logic
- Combine via sessions for complete protocol flows
- Use hierarchical sessions for multi-protocol operations

## Session Types and Protocol Choreography

Valence implements session types for DeFi protocols. Session types are a type system for communication protocols that ensure messages are exchanged in the correct order.

### Core Concepts

- **Session**: Defines a multi-party protocol with linear execution
- **Accounts**: Participants with specific roles in the protocol
- **State Machine**: Enforces the protocol's communication patterns

### Session Structure

```rust
pub struct Session {
    pub bump: u8,
    pub owner: Pubkey,
    pub accounts: Vec<Pubkey>,          // Session accounts (up to 16)
    pub consumed: bool,                 // Move semantics - consumed after move
    pub created_at: i64,
    pub protocol_type: [u8; 32],        // Hash identifying the protocol choreography
    
    // Verification context fields
    pub verification_data: [u8; 256],   // Shared state between verifiers
    pub parent_session: Option<Pubkey>, // For composable sessions
}
```

Sessions implement move semantics for type linearity:
```rust
// Session can be moved but not copied
move_session(session, new_owner);
// session.consumed = true
// Original owner can no longer use
```

### Account Structure

```rust
pub struct SessionAccount {
    pub bump: u8,
    pub session: Pubkey,            // Parent session
    pub verifier: Pubkey,           // Authorization logic
    
    // Security fields
    pub nonce: u64,                 // Replay protection
    
    // Lifecycle fields
    pub uses: u32,                  // Usage count
    pub max_uses: u32,              // Maximum allowed uses
    pub expires_at: i64,            // Lifetime expiration
    pub created_at: i64,
    
    // Metadata
    pub metadata: [u8; 64],         // Verifier-specific data
}
```

## Verification Model

The Valence microkernel delegates all authorization logic to verifier programs, maintaining simplicity in the core:

### Verification Flow

1. **Account Creation**: Specify verifier program that implements authorization logic
2. **Operation Request**: Caller provides operation data to use account
3. **Verifier Check**: Core delegates to verifier via CPI
4. **Usage Update**: If verified, increment usage count and nonce

### Verifier Interface

```rust
// Verifier programs implement this interface
pub fn verify_operation(
    account: &SessionAccount,
    caller: &Pubkey,
    operation_data: &[u8],
    session_data: &[u8; 256], // Shared session context
) -> Result<()> {
    // Implement any authorization logic:
    // - Check ownership
    // - Validate balances
    // - Verify time conditions
    // - Check external state
    // - Enforce protocol rules
}
```

### Example Usage

```rust
// Simple ownership verifier
if caller != expected_owner {
    return Err(Error::Unauthorized);
}

// Complex DeFi verifier might check:
// - Collateral ratios from oracle
// - Liquidation thresholds
// - Protocol parameters
// - Multi-sig requirements
```

## Core Instructions

### Session Management
- `create_session`: Create new session container (optionally as child of another)
- `move_session`: Transfer ownership (consumes session)
- `update_session_data`: Update shared verification data
- `cleanup_session`: Remove expired accounts
- `close_session`: Close consumed session and return rent

### Account Management
- `add_account`: Add account with verifier program
- `use_account`: Use account with verifier authorization
- `use_accounts_atomic`: Use multiple accounts atomically (all or nothing)
- `use_accounts_atomic_with_depth`: CPI depth-aware version with verification modes
- `use_accounts_simple`: Use two accounts atomically (simplified version)
- `use_account_if`: Conditionally use account based on state
- `create_account_with_session`: Helper for single-account sessions
- `update_account_metadata`: Store protocol data (up to 64 bytes)
- `close_account`: Close expired account and return rent

### Shard Management
- `deploy_shard`: Deploy code/state definition with integrity hash
- `execute_shard`: Execute shard with session account authorization

## Session Composability and Verification Context

Sessions serve dual purposes:
1. **Protocol Choreography**: Define valid state transitions (session types)
2. **Verification Context**: Share data between verification function (256 bytes)

### Verification Data Sharing

Each session has a 256-byte `verification_data` field that verifiers can read and write:

```rust
// First verifier: Oracle writes price
verify_price(oracle_account) -> writes [price, timestamp, confidence]

// Second verifier: Lending reads price  
verify_collateral(collateral_account) -> reads price from session
```

### Verification Cache Layout

For CPI optimization, the verification_data can store cached results:

```
[0..4]:   Cache version/flags
[4..8]:   Timestamp (5-minute TTL)
[8..40]:  Bitmap (256 accounts verified)
[40..256]: Verifier-specific data
```

### Hierarchical Sessions

Sessions can have parent sessions, creating a hierarchy:

```rust
// Create parent session
let parent = create_session(
    protocol_type: "AGGREGATOR",
    parent_session: None,
);

// Create child sessions
let child1 = create_session(
    protocol_type: "LENDING",
    parent_session: Some(parent),
);
```

Benefits:
- **Scoped Contexts**: Each protocol has its own session
- **Shared Parent State**: Children can read parent's verification_data
- **Atomic Multi-Protocol Ops**: Parent coordinates children
- **Clean Separation**: Each child manages its own accounts

## Multi-Account Atomic Operations

The `use_accounts_atomic` instruction implements a 3-phase commit protocol that ensures atomicity across multiple account operations. This design prevents partial execution and maintains consistency.

### Phase 1: Validation (Read-Only)
```rust
// Deserialize and validate all accounts without any state changes
for account_info in remaining_accounts {
    // Borrow account data (read-only)
    let account_data = account_info.try_borrow_data()?;
    
    // Deserialize to verify structure
    let session_account = SessionAccount::try_deserialize(&account_data[8..])?;
    
    // Check account is not expired
    require!(clock.unix_timestamp < session_account.expires_at);
    
    // Store for later use (no modifications yet)
    session_accounts.push(session_account);
}
```

**Purpose**: 
- Ensure all accounts are valid before any operations
- Detect malformed accounts early
- Verify all accounts are within their lifetime
- No state changes occur - completely read-only

### Phase 2: Verification (External Calls)
```rust
// Call each account's verifier via CPI
for (i, account_info) in remaining_accounts.enumerate() {
    let cpi_ctx = CpiContext::new(
        verifier_program,
        VerifyAccount {
            account: account_info,
            caller: caller,
        }
    );
    
    // If ANY verifier fails, entire operation aborts
    verify_account(cpi_ctx).map_err(|e| {
        msg!("Verification failed for account {}: {:?}", i, e);
        e
    })?;
}
```

**Purpose**:
- Execute authorization logic for each account
- Verifiers can check complex conditions
- Verifiers can read/write session verification_data
- All verifiers must succeed or none proceed

### Phase 3: State Update (Atomic Write)
```rust
// Only reached if ALL validations and verifications passed
for account_info in remaining_accounts {
    // Now borrow mutably for writing
    let mut account_data = account_info.try_borrow_mut_data()?;
    
    // Deserialize again (required for mutable access)
    let mut session_account = SessionAccount::try_deserialize(&account_data[8..])?;
    
    // Update state
    session_account.uses += 1;
    
    // Serialize back
    session_account.try_serialize(&mut account_data[8..])?;
}
```

**Purpose**:
- Apply state changes to all accounts
- Increment usage counters
- Update any other mutable fields

### Atomicity Guarantees

The 3-phase design provides several guarantees:

1. **All-or-Nothing**: Either all accounts are used successfully, or none are
2. **No Partial States**: Cannot have some accounts verified but not others
3. **Consistent View**: All verifiers see the same initial state
4. **Ordered Execution**: Phases execute strictly in sequence
5. **Failure Recovery**: Any failure rolls back entire transaction

## CPI Depth Optimization

Solana enforces a maximum Cross-Program Invocation (CPI) depth of 4. This creates a challenge for multi-account atomic operations, which need to verify each account through its respective verifier program.

### The Problem

Without optimization:
- Initial call to Valence: depth 1
- Call to `use_accounts_atomic`: depth 2  
- Each verifier CPI: depth 3
- Any nested calls: depth 4 (limit reached)

With 16 accounts maximum, naive implementation would require 16 separate CPIs at depth 3, making complex protocols impossible.

### The Solution: Multi-Mode Verification

#### Verification Modes

```rust
pub enum VerificationMode {
    Direct,  // Traditional CPI (for complex verifiers)
    Batch,   // Single CPI for multiple accounts
    Inline,  // No CPI needed (simple patterns)
    Cached,  // Use previous verification result
}
```

#### Inline Verification

For common patterns, verification happens directly in the core without CPI:

```rust
pub enum InlineVerifierType {
    SimpleOwner,    // owner == caller
    TimeBased,      // current_time < expiry
    CounterBased,   // uses < max_uses
    Standard,       // All of the above
}
```

Benefits:
- Zero CPI cost
- Perfect for simple authorization
- Maintains security through pattern matching

#### Batch Verification

Verifiers can implement a batch interface to verify multiple accounts in one CPI:

```rust
// Instead of:
for account in accounts {
    verify_account(account)?; // N CPIs
}

// We do:
verify_accounts_batch(accounts)?; // 1 CPI
```

Benefits:
- O(1) CPI cost instead of O(n)
- Verifier can optimize validation logic
- Shared context through session data

#### Depth-Aware Execution

The system adapts based on available CPI depth:

```rust
match remaining_depth {
    0..=1 => Full functionality with all modes,
    2 => Prefer batch verification,
    3 => Only inline or cached verification,
    4 => Error: insufficient depth,
}
```

### Implementation Example

```rust
pub fn use_accounts_atomic_with_depth(
    ctx: Context<UseAccountsAtomicOptimized>,
    estimated_depth: u8,
) -> Result<()> {
    // Phase 1: Classify accounts by verification type
    let inline_accounts = filter_inline_verifiers(&accounts);
    let cached_accounts = filter_cached_valid(&accounts);
    let remaining = group_by_verifier(&accounts);
    
    // Phase 2: Optimize verification order
    // - Inline: 0 CPIs
    // - Cached: 0 CPIs  
    // - Batch: 1 CPI per verifier
    // - Direct: 1 CPI per account
    
    // Phase 3: Execute with depth awareness
    verify_inline_accounts(&inline_accounts)?;
    skip_cached_accounts(&cached_accounts);
    verify_remaining_optimally(&remaining, depth)?;
}
```

### Performance Comparison

| Scenario | Naive CPIs | Optimized CPIs | Improvement |
|----------|------------|----------------|-------------|
| 2 simple accounts | 2 | 0 (inline) | 100% |
| 8 same verifier | 8 | 1 (batch) | 87.5% |
| 4 cached accounts | 4 | 0 (cached) | 100% |
| Mixed 16 accounts | 16 | 2-4 | 75-87.5% |

## Usage Patterns

### Session Types Examples

#### Flash Loan Protocol
```rust
// Flash loan protocol session
let flash_session = create_session(borrower, hash("FLASH_LOAN"));

// Flash loan account with verifier enforcement
let flash_account = add_account(
    session: flash_session,
    verifier: flash_verifier,
    max_uses: 3, // Borrow, execute, repay
    lifetime: 300, // 5 minute flash loan
);

// Verifier enforces protocol ordering
execute_flash_loan(flash_account); // Verifier ensures correct sequence
```

#### Cross-Margin Trading
```rust
// Session choreographs multiple accounts across pools
let margin_session = create_session(trader, hash("CROSS_MARGIN"));

// Collateral account in ETH pool
let eth_account = add_account(
    session: margin_session,
    verifier: eth_pool_verifier,
    max_uses: 100, // Many operations allowed
    lifetime: 86400 * 30, // 30 days
    metadata: encode_role("collateral"),
);

// Debt account in USDC pool  
let usdc_account = add_account(
    session: margin_session,
    verifier: usdc_pool_verifier,
    max_uses: 100, // Many operations allowed
    lifetime: 86400 * 30, // 30 days
    metadata: encode_role("debt"),
);

// Session ensures atomic execution across pools
use_accounts_atomic([eth_account, usdc_account]);
```

### Conditional Account Usage

Four condition types are supported:

```rust
// Type 0: Use if usage count less than value
use_account_if(account, 0, 5); // uses < 5

// Type 1: Use if age less than value (seconds)
use_account_if(account, 1, 3600); // age < 3600 seconds

// Type 2: Use if usage count equals exact value  
use_account_if(account, 2, 3); // uses == 3

// Type 3: Use if metadata first 8 bytes match value
use_account_if(account, 3, encoded_target_value); // metadata[0..8] == value
```

### Optimized Multi-Account Usage

For CPI depth-constrained environments:

```rust
// Tell Valence your current depth
use_accounts_atomic_with_depth(ctx, current_depth)?;

// Use simple verifiers when possible
let verifier = create_simple_owner_verifier(); // Inline-able

// Implement batch verification in your verifier
pub fn verify_batch(ctx: Context<VerifyBatch>) -> Result<()> {
    for account in ctx.remaining_accounts {
        // Verify all atomically
    }
}
```

## Verification Model

### Verifier Pattern

All authorization policies live in user-deployed verifiers. The kernel makes no policy decisions.

#### Simple Verifier
```rust
pub fn verify_account(ctx: Context<Verify>) -> Result<()> {
    let account = ctx.accounts.account;
    let caller = ctx.accounts.caller;
    
    // Check operation is allowed
    require!(
        verify_operation(&account, &caller, &operation_data)?,
        Error::Unauthorized
    );
    
    // Verify caller is authorized
    require!(
        caller.key() == account.metadata[..32], // Owner stored in metadata
        Error::Unauthorized
    );
    
    Ok(())
}
```

#### Verifier with Session Context
```rust
pub fn verify_with_session(ctx: Context<VerifyWithSession>) -> Result<()> {
    let session = &mut ctx.accounts.session;
    
    // Read shared data
    let price = u64::from_le_bytes(
        session.verification_data[0..8].try_into()?
    );
    
    // Perform verification using shared data
    verify_collateral_value(price)?;
    
    // Write result back
    session.verification_data[8..16].copy_from_slice(
        &collateral_value.to_le_bytes()
    );
    
    Ok(())
}
```

### Verifier Development Guidelines

#### For Simple Patterns
- Use recognized inline patterns when possible
- Store owner in first 32 bytes of metadata
- Keep logic stateless for caching

#### For Complex Verifiers
- Implement batch interface for efficiency
- Use session data for shared context
- Document time-bound validity

#### Security Considerations
- **Inline Verification**: Only well-known, audited patterns
- **Cache Security**: TTL prevents stale data, session-scoped
- **Batch Security**: Must validate all-or-nothing

## Security Model

### Trust Boundaries

**Trusted (Kernel):**
- valence-core: Session/account management (~1000 lines)
- State machine verification logic
- Inline verification patterns

**Untrusted (Userspace):**
- Verifiers: User authorization logic
- Business logic: User protocols

### Security Properties

1. **Verifier Isolation**: Authorization logic cannot modify kernel state
2. **Replay Protection**: Nonces prevent operation replay
3. **Move Semantics**: Sessions prevent double-spending
4. **Usage Limits**: Max uses prevent unbounded operations
5. **Time Bounds**: Expiration prevents stale accounts
6. **Overflow Protection**: All arithmetic uses checked operations
7. **3-Phase Atomic Operations**: Validate → verify → execute pattern
8. **Hierarchical Security**: Child sessions require parent access
9. **CPI Depth Protection**: Adaptive verification modes

### Attack Prevention

- **Authorization Bypass**: Mandatory CPI to verifier program
- **Replay Attack**: Nonce tracking prevents replay
- **Session Forgery**: PDA derivation with program ID
- **Usage Exhaustion**: Max uses limit enforced
- **Time-based Attacks**: Expiration timestamps checked
- **Parameter Tampering**: Immutable account data
- **CPI Exhaustion**: Multi-mode verification prevents depth attacks

## Design Trade-offs

### What We Optimize For

1. **Simplicity**: < 1000 lines for core
2. **Security**: Minimal attack surface with cryptographic guarantees
3. **Flexibility**: Policy in userspace
4. **Performance**: Zero-copy, minimal allocations, CPI optimization
5. **Composability**: Hierarchical sessions and shared context

### What We Don't Provide

1. **Built-in Policies**: No default authorization schemes
2. **State Management**: No global registries
3. **Complex Types**: No dynamic structures (except accounts list)
4. **Backwards Compatibility**: Clean design over legacy support

## Data Layout Conventions

Since verification_data is untyped, protocols should establish conventions:

### Standard Layouts

```rust
// Price data (56 bytes)
[price: u64][timestamp: i64][confidence: u64][source: Pubkey]

// Position data (32 bytes)
[collateral: u64][debt: u64][health: u64][update_time: i64]

// Simple value (8 bytes)
[value: u64]

// Verification cache (see Verification Cache Layout above)
```

### Best Practices

1. **Define Clear Layouts**: Document your data layout
2. **Version Your Formats**: Include version byte if needed
3. **Validate Parent Access**: Check parent_session is expected
4. **Use Atomic Operations**: Update all related data together
5. **Include Timestamps**: For time-sensitive data
6. **Reserve Space**: Leave room for future fields

## Future Enhancements

1. **Verifier Registry**: On-chain registry of inline patterns
2. **Parallel Verification**: Use Solana's parallel execution
3. **Zero-Knowledge Proofs**: Verify without CPI
4. **Compression**: Pack multiple verifications into one

## Conclusion

Valence provides a minimal, secure foundation for building complex DeFi protocols on Solana. By combining verifier-based authorization, session types, hierarchical composition, and CPI depth optimization, it enables sophisticated multi-protocol operations while maintaining simplicity and security.

The kernel remains intentionally minimal, with innovation happening in verifiers and protocols. This design ensures a stable core that rarely changes while supporting the evolution of DeFi primitives above it.