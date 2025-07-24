# Valence Developer Guide

## Getting Started

This guide helps you build applications on Valence, from simple authorization to complex DeFi protocols.

### Prerequisites

- Rust 1.75+
- Solana CLI 1.18+
- Anchor 0.31+

### Quick Start

```bash
# Clone the repository
git clone https://github.com/timewave-computer/valence-solana
cd valence-solana

# Build programs
anchor build

# Run tests
anchor test
```

## Core Concepts

### 1. Sessions and Accounts

Sessions manage accounts, accounts provide authorization:

```rust
// Create a session (container for accounts)
let session = create_session(owner);

// Add accounts with different verifiers
let account1 = add_account(session, owner_verifier, params, 1_hour);
let account2 = add_account(session, multisig_verifier, params, 24_hours);

// Use accounts multiple times
use_account(account1); // uses: 1
use_account(account1); // uses: 2
```

### 2. Verifiers (Authorization Logic)

Verifiers implement your authorization policies:

```rust
#[program]
pub mod my_verifier {
    pub fn verify_account(ctx: Context<Verify>) -> Result<()> {
        // Access account directly from context
        let account = &ctx.accounts.account;
        
        // Your authorization logic
        let owner = Pubkey::try_from(&account.metadata[..32])?;
        require_keys_eq!(ctx.accounts.caller.key(), owner);
        
        Ok(())
    }
}
```

### 3. Shards (Business Logic)

Shards contain executable code:

```rust
pub fn my_business_logic(input: &[u8]) -> Result<Vec<u8>> {
    // Process input
    let amount = u64::from_le_bytes(input[..8].try_into()?);
    
    // Use extensions for common operations
    let result = valence_extensions::liquidity::calculate_swap(amount)?;
    
    Ok(result.to_le_bytes().to_vec())
}
```

## Building Your First Application

### Step 1: Design Your Authorization

```rust
// Simple owner-only verifier (see valence-extensions/examples/owner_verifier.rs)
pub fn verify_account(ctx: Context<Verify>) -> Result<()> {
    let account = parse_account(ctx.accounts.account)?;
    let owner = Pubkey::from(&account.params[..32]);
    
    require_keys_eq!(
        ctx.accounts.caller.key(),
        owner,
        ErrorCode::Unauthorized
    );
    
    Ok(())
}
```

### Step 2: Create Session and Account

```rust
// Deploy verifier
let verifier_id = deploy_program("my_verifier.so")?;

// Create session
let session_id = Pubkey::new_unique();
valence_core::create_session(ctx, session_id)?;

// Add account with owner in params
let mut params = [0u8; 256];
params[..32].copy_from_slice(&owner.to_bytes());

valence_core::add_account(
    ctx,
    verifier_id,
    params.to_vec(),
    3600, // 1 hour lifetime
)?;
```

### Step 3: Execute with Authorization

```rust
// Use account (verifies + increments usage)
valence_core::use_account(ctx)?;

// Or execute shard with account
valence_shard::execute(ctx, input)?;
```

## Advanced Patterns

### Linear Operations (Lending Protocol)

The counter system creates linear types where operations must follow a specific order:

```rust
// 1. Create account with linear lending verifier
let account = add_account(session, linear_lending_verifier, params, 24_hours);

// 2. Operations MUST follow linear flow
deposit(account, amount)?;              // OK: counter 0→1
transfer_voucher(account, voucher)?;    // OK: counter 1→2
withdraw(account, amount)?;             // OK: counter 2→3

// Invalid flows are rejected by verifier
deposit(account, amount)?;              // OK: counter 0→1
withdraw(account, amount)?;             // ERROR: counter 1 expects transfer!
```

The verifier enforces this through the counter:
```rust
pub fn verify_linear_flow(ctx: Context<Verify>) -> Result<()> {
    let account = &ctx.accounts.account;
    let operation = parse_operation(ctx.remaining_accounts)?;
    
    // Linear type system using counter
    match (account.uses, operation) {
        (0, DEPOSIT) => Ok(()),     // First operation must be deposit
        (1, TRANSFER) => Ok(()),    // Second must be transfer  
        (2, WITHDRAW) => Ok(()),    // Third must be withdraw
        _ => Err(InvalidOrder),     // No other transitions allowed
    }
}
```

This creates true linear types:
- Each operation consumes the previous state
- Counter prevents skipping or replaying operations
- Session move transfers the entire linear flow

### Complex Verifiers

```rust
// Multi-signature with time window
pub fn verify_multisig_timelock(ctx: Context<Verify>) -> Result<()> {
    let account = &ctx.accounts.account;
    
    // Check time window (stored in metadata)
    let start = i64::from_le_bytes(account.metadata[0..8].try_into()?);
    let end = i64::from_le_bytes(account.metadata[8..16].try_into()?);
    let now = Clock::get()?.unix_timestamp;
    require!(now >= start && now <= end, Error::OutsideWindow);
    
    // Check signatures (2 of 3) - signers passed in remaining accounts
    // In real implementation, signer keys would be stored off-chain
    // or in a separate account
    
    let mut valid_sigs = 0;
    for signer in ctx.remaining_accounts {
        if signer.is_signer {
            if signer.key() == signer1 || 
               signer.key() == signer2 || 
               signer.key() == signer3 {
                valid_sigs += 1;
            }
        }
    }
    
    require!(valid_sigs >= 2, Error::InsufficientSigners);
    Ok(())
}
```

### Using Extensions

```rust
use valence_extensions::{math, events, batching};
use valence_extensions::math::FixedPoint;

// Math operations with fixed-point arithmetic
let reserve_a_fp = FixedPoint::from_int(reserve_a);
let reserve_b_fp = FixedPoint::from_int(reserve_b);
let amount_in_fp = FixedPoint::from_int(amount_in);

// Calculate using built-in math operations
let output = calculate_swap_output(reserve_a_fp, reserve_b_fp, amount_in_fp)?;

// Event emission
events::emit_event!("Swap", 
    user: ctx.accounts.user.key(),
    amount_in: amount_in,
    amount_out: output.to_int()
);

// Batch operations
batching::Batch::new()
    .add(|| transfer(from, to, amount))
    .add(|| update_price(new_price))
    .execute_atomic()?;
```

## Best Practices

### 1. Verifier Design

- **Keep Simple**: One responsibility per verifier
- **Encode Policies**: All rules in parameters
- **Avoid State**: Verifiers should be stateless
- **Test Thoroughly**: Edge cases matter

### 2. Account Metadata

Use the 64-byte metadata field wisely:

```rust
// Good: Store essential references
account.metadata = voucher_id.to_bytes();

// Good: Pack multiple small values
let mut metadata = [0u8; 64];
metadata[0..8] = last_price.to_le_bytes();
metadata[8..16] = last_update.to_le_bytes();
metadata[16..48] = position_id.to_bytes();
metadata[48..64] = path_info;
```

### 3. Error Handling

```rust
#[error_code]
pub enum MyError {
    #[msg("Clear, actionable error message")]
    Unauthorized,
    
    #[msg("Amount {} exceeds limit {}", .amount, .limit)]
    AmountExceeded { amount: u64, limit: u64 },
}
```

### 4. Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_authorization() {
        // Test successful case
        let result = verify_owner(&valid_context);
        assert!(result.is_ok());
        
        // Test failure case
        let result = verify_owner(&invalid_context);
        assert_eq!(result.err(), Some(Error::Unauthorized));
    }
}
```

## Common Patterns

### Rate Limiting

```rust
pub fn verify_rate_limited(ctx: Context<Verify>) -> Result<()> {
    let account = &ctx.accounts.account;
    
    // Store last use time in metadata
    let last_use = i64::from_le_bytes(account.metadata[0..8].try_into()?);
    let min_interval = i64::from_le_bytes(account.metadata[8..16].try_into()?);
    
    let now = Clock::get()?.unix_timestamp;
    require!(
        now - last_use >= min_interval,
        Error::TooFrequent
    );
    
    Ok(())
}
```

### Conditional Operations

```rust
pub fn verify_conditional(ctx: Context<Verify>) -> Result<()> {
    let account = &ctx.accounts.account;
    
    // Check account has voucher
    let voucher = Pubkey::try_from(&account.metadata)?;
    require!(voucher != Pubkey::default(), Error::NoVoucher);
    
    // Check external condition
    let oracle_price = get_oracle_price()?;
    let min_price = u64::from_le_bytes(account.metadata[0..8].try_into()?);
    require!(oracle_price >= min_price, Error::PriceTooLow);
    
    Ok(())
}
```

## Troubleshooting

### Common Issues

1. **"Session already consumed"**
   - Sessions can only be moved once
   - Create new session after move

2. **"Account expired"**
   - Accounts have time-based expiry
   - Create with longer lifetime or new account

3. **"Verification failed"**
   - Check verifier logic
   - Ensure params encoded correctly
   - Verify caller matches requirements

### Debugging Tips

```rust
// Add logging to verifiers
msg!("Verifying account {} for caller {}", 
     account.key(), 
     caller.key());

// Check account state
msg!("Account uses: {}, expires: {}", 
     account.uses, 
     account.expires_at);

// Validate metadata
msg!("First 32 bytes of metadata: {:?}", 
     &account.metadata[..32]);
```

## Migration Guide

Coming from traditional Solana programs:

1. **Authorization**: Move checks to verifiers
2. **State**: Use protocol programs, not Valence
3. **Composition**: Combine verifiers and shards
4. **Testing**: Test verifiers independently

## Project Structure

```
valence-solana/
programs/                    # All programs and libraries
    valence-core/           # Core kernel (566 lines)
    valence-shard/          # Shard execution (160 lines)
    valence-extensions/     # Optional helpers and utilities
        src/
            math.rs         # Fixed-point math library (feature-flagged)
            events.rs       # Event patterns (feature-flagged)
            batching.rs     # Batch execution (feature-flagged)
        examples/           # Verifier examples
            owner_verifier.rs
            linear_lending_verifier.rs
            multidimensional_curve_verifier.rs
examples/                    # Integration examples
    session-accounts-demo/  # Full workflow demo
    lending-protocol-demo/  # Multi-use accounts
    shard-swap/            # Shard example
docs/                       # Documentation
    architecture.md         # System design
    developer-guide.md      # This guide
    security-model.md       # Security analysis
```

## Resources

- [Architecture Documentation](./architecture.md)
- [Security Model](./security-model.md)
- [Example Verifiers](../programs/valence-extensions/src/examples/)
- [Example Programs](../examples/)

Remember: Valence provides mechanisms, you provide policies!