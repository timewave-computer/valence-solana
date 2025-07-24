# Valence Security Model

## Overview

Valence's security model is based on minimal trusted code, clear trust boundaries, and user-controlled authorization. All policy decisions are delegated to user-deployed programs.

## Threat Model

### In Scope

- Unauthorized account usage
- Session replay or hijacking
- Parameter tampering
- Resource exhaustion
- Verification bypass
- Code injection in shards
- Double-spending sessions

### Out of Scope

- Vulnerabilities in user verifiers
- Business logic errors
- Solana runtime bugs
- Network attacks
- Social engineering

## Trust Boundaries

### Trusted Components

**valence-core**
- Creates sessions with integrity
- Manages account lifecycle
- Enforces move semantics
- Delegates to verifiers
- Tracks usage and metadata
- Includes shard deployment and execution
- Implements atomic multi-account operations

### Untrusted Components

**Verifiers**
- User-deployed
- Implement policies
- Isolated execution
- No kernel access

**Shards**
- User business logic
- Sandboxed execution
- No special privileges

## Security Properties

### 1. Session Integrity

```rust
pub struct Session {
    pub owner: Pubkey,       // Fixed at creation
    pub accounts: Vec<...>,  // Append-only
    pub consumed: bool,      // Monotonic: false→true
    pub created_at: i64,     // Immutable
}
```

**Guarantees:**
- Owner cannot change
- Accounts only added, never removed
- Consumption is irreversible
- Creation time fixed

### 2. Account Security

```rust
pub struct SessionAccount {
    pub session: Pubkey,     // Parent session reference
    pub verifier: Pubkey,    // Fixed at creation
    pub nonce: u64,          // Replay protection
    pub uses: u32,           // Monotonic increase
    pub max_uses: u32,       // Usage limit
    pub expires_at: i64,     // Fixed at creation
    pub created_at: i64,     // Creation timestamp
    pub metadata: [u8; 64],  // Protocol-controlled
}
```

**Properties:**
- Session reference immutable
- Verifier immutable
- Nonce monotonically increases
- Usage only increases up to max_uses
- Time-based expiry enforced
- Metadata updateable by authorized parties

### 3. Authorization Model

The verifier-based system provides flexible authorization:

```rust
// Session move semantics
session.consumed = false;
move_session(session, Bob);
session.consumed = true;

// Verifier enforces authorization logic
account.verifier = custom_verifier;
use_account(operation_data);     // Verifier decides if allowed
```

**Security Properties:**
- Verifiers implement all authorization logic
- Nonce prevents replay attacks
- Usage limits prevent unbounded operations
- Time bounds prevent stale accounts

**Example Attack Prevention:**
```rust
// Attacker tries to replay operation
account.nonce = 5;
operation_data = [4, 0, 0, 0, 0, 0, 0, 0, ...];  // Old nonce (4)
use_account(operation_data) → ERROR  // Invalid nonce

// Attacker tries to exceed limits
account.uses = account.max_uses;
use_account(operation_data) → ERROR  // Account expired (usage limit)
```

### 4. Verification Isolation

Verifiers run in separate programs:
- Read-only session view
- Cannot modify kernel
- Return success/failure only
- No access to other sessions

## Attack Analysis

### 1. Session Forgery

**Attack**: Create fake session
**Defense**: PDA with program ID

```rust
seeds = [SESSION_SEED, session_id]
// Only valence-core can create
```

### 2. Account Hijacking

**Attack**: Use someone else's account
**Defense**: Verifier authorization

```rust
// Every use requires verification
valence_core::use_account()
  → CPI to verifier
  → Check authorization
  → Success/failure
```

### 3. Parameter Tampering

**Attack**: Modify params after creation
**Defense**: Immutable account data

```rust
#[account(mut)] // For usage counter and nonce
pub account: Account<SessionAccount>
// verifier and expiration fields never modified
```

### 4. Replay Attack

**Attack**: Reuse consumed session
**Defense**: Move semantics

```rust
require!(!session.consumed, Error::AlreadyConsumed);
// Once true, permanently unusable
```

### 5. Code Injection

**Attack**: Execute malicious code
**Defense**: Hash verification

```rust
pub struct Shard {
    pub code_hash: [u8; 32],
}
// Hash checked before execution
```

### 6. Usage Overflow

**Attack**: Overflow usage counter
**Defense**: Checked arithmetic

```rust
account.uses = account.uses
    .checked_add(1)
    .ok_or(Error::Overflow)?;
```

## Verifier Security

### Secure Verifier Template

```rust
pub fn verify_account(ctx: Context<Verify>) -> Result<()> {
    // 1. Parse safely
    let account = SessionAccount::try_deserialize(
        &mut &ctx.accounts.account.data.borrow()[8..]
    )?;
    
    // 2. Validate metadata if needed
    require!(account.metadata.len() == 64, Error::Invalid);
    
    // 3. Check authorization
    let authorized = check_authorization(&account, &ctx.accounts.caller);
    require!(authorized, Error::Unauthorized);
    
    // 4. No side effects
    // Don't modify any state
    
    Ok(())
}
```

### Common Vulnerabilities

**1. Integer Overflow**
```rust
// Bad
let total = a + b;

// Good
let total = a.checked_add(b).ok_or(Error::Overflow)?;
```

**2. Unchecked Arrays**
```rust
// Bad
let value = metadata[index];

// Good
let value = metadata.get(index).ok_or(Error::OutOfBounds)?;
```

**3. Missing Validation**
```rust
// Bad
let key = Pubkey::from(bytes);

// Good
let key = Pubkey::try_from(bytes)
    .map_err(|_| Error::InvalidPubkey)?;
```

## Operational Security

### Deployment

1. **Verify Programs**
```bash
anchor verify-id <program-id>
solana program dump <program-id> program.so
```

2. **Make Immutable**
```bash
solana program set-upgrade-authority <program-id> --final
```

3. **Audit Trail**
```rust
msg!("Session {} created by {}", session_id, owner);
msg!("Account {} used, count: {}", account_key, uses);
```

### Best Practices

**1. Session Management**
- Create close to use
- Use unique IDs
- Set appropriate lifetimes
- Clean up expired accounts

**2. Metadata Encoding**
```rust
// Fixed layout for metadata (64 bytes)
metadata[0..32] = owner.to_bytes();
metadata[32..40] = amount.to_le_bytes();
metadata[40..48] = timestamp.to_le_bytes();
// Remaining bytes for protocol-specific data
```

**3. Metadata Usage**
```rust
// Store critical references
metadata = voucher_id.to_bytes();

// Don't store sensitive data
// metadata != private_key
```

## Incident Response

### If Compromised

**Session Level:**
1. Cannot revoke (immutable)
2. Wait for expiry
3. Create new sessions

**Verifier Level:**
1. Deploy patched verifier
2. Create new accounts
3. Migrate users

**Kernel Level:**
1. Requires program upgrade
2. All sessions invalid
3. Full migration needed

### Monitoring

Track key metrics:
- Session creation rate
- Account usage patterns
- Verification failures
- Expired accounts

## Security Assumptions

1. **Solana Runtime**: BPF sandbox secure
2. **Cryptography**: Ed25519, SHA256 secure
3. **Economics**: Rent prevents spam
4. **Time**: Clock manipulation limited
5. **Programs**: Deployed code correct

## Linear Type Security

The counter-based linear type system provides additional security:

### Attack: Operation Reordering
```rust
// Attacker tries to withdraw before deposit
account.uses = 0;
withdraw() → ERROR: Counter 0 only allows deposit
```

### Attack: Operation Replay
```rust
// Attacker tries to deposit twice
deposit();  // uses: 0→1
deposit();  // ERROR: Counter 1 doesn't allow deposit
```

### Attack: State Branching
```rust
// Linear flow prevents state explosion
// Each counter value maps to exactly one valid operation
// No branching paths possible
```

## Audit Checklist

### Core Program
- [ ] PDA seeds include program ID
- [ ] All arithmetic checked
- [ ] State transitions valid
- [ ] No unnecessary mutability
- [ ] Proper error handling
- [ ] Counter increments are atomic

### Verifiers
- [ ] Parameter validation
- [ ] No external mutations
- [ ] Deterministic logic
- [ ] Proper error codes
- [ ] No reentrancy
- [ ] Linear flow enforcement

### Integration
- [ ] Verify program IDs
- [ ] Check account ownership
- [ ] Validate all inputs
- [ ] Handle all errors
- [ ] Test edge cases
- [ ] Test invalid operation orders

## Conclusion

Valence's security emerges from:
- **Minimal kernel**: Core logic in focused modules
- **Clear boundaries**: Kernel vs userspace
- **Simple invariants**: Immutable, monotonic
- **User control**: Policies in verifiers
- **Economic limits**: Rent and compute

The microkernel approach means most vulnerabilities live in userspace verifiers, not the trusted core. All authorization logic is delegated to external programs, keeping the kernel minimal and focused on mechanism rather than policy.