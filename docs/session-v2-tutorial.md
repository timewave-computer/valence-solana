# Session V2 Developer Tutorial

## Introduction

Welcome to Valence Session V2! This tutorial will get you building with Sessions in under an hour. **Sessions are the only concept you need to understand** - we've hidden all the complex infrastructure details for you.

## What You'll Build

By the end of this tutorial, you'll have built a simple token voting system that demonstrates all key Session V2 concepts:
- Creating sessions with capabilities
- Executing single operations  
- Building and executing bundles
- Error handling

## Prerequisites

- Basic Rust and Anchor knowledge
- Valence development environment set up
- 30-60 minutes

## Step 1: Understanding Sessions

A **Session** is your main tool for interacting with Valence. Think of it as a secure container that:
- Holds **capabilities** (what you can do)
- Tracks **state** (your application data)
- Manages **operations** (functions you execute)

```rust
// This is all you need to think about - no account complexity!
use valence_shard::{Session, Capability, create_session_v2};
```

## Step 2: Your First Session

Let's create a simple voting session:

```rust
use anchor_lang::prelude::*;
use valence_shard::*;

#[program]
pub mod voting_app {
    use super::*;

    pub fn create_voting_session(
        ctx: Context<CreateVotingSession>,
        proposal_name: String,
    ) -> Result<()> {
        // Step 1: Define what your session can do
        let mut capabilities = Capabilities::none();
        capabilities.add(Capability::Read);    // Read vote counts
        capabilities.add(Capability::Write);   // Record votes
        
        // Step 2: Set initial state  
        let initial_state = VotingState {
            proposal_name: proposal_name.clone(),
            yes_votes: 0,
            no_votes: 0,
            total_voters: 0,
        };
        let state_bytes = anchor_lang::AnchorSerialize::try_to_vec(&initial_state)?;
        
        // Step 3: Create the session - that's it!
        let session = create_session_v2(
            ctx.accounts.session_ctx,
            capabilities.0,              // Your capabilities
            state_bytes,                 // Initial data
            "voting-session".to_string(), // Namespace
            1,                           // Nonce
            vec![]                       // Metadata
        )?;
        
        msg!("Created voting session for: {}", proposal_name);
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct VotingState {
    pub proposal_name: String,
    pub yes_votes: u32,
    pub no_votes: u32,
    pub total_voters: u32,
}
```

**Key Points:**
- No account complexity - just specify capabilities and state
- Capabilities define what operations are allowed
- State is your application data

## Step 3: Executing Operations

Now let's add voting functionality:

```rust
pub fn cast_vote(
    ctx: Context<CastVote>,
    vote_yes: bool,
) -> Result<()> {
    // Create vote data
    let vote_data = VoteData {
        voter: ctx.accounts.voter.key(),
        vote_yes,
        timestamp: Clock::get()?.unix_timestamp,
    };
    
    let args = anchor_lang::AnchorSerialize::try_to_vec(&vote_data)?;
    
    // Execute vote - automatic capability checking!
    execute_on_session(
        ctx.accounts.session_ctx,
        VOTE_FUNCTION_HASH,
        args
    )?;
    
    msg!("Vote cast: {}", if vote_yes { "YES" } else { "NO" });
    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct VoteData {
    pub voter: Pubkey,
    pub vote_yes: bool,
    pub timestamp: i64,
}

const VOTE_FUNCTION_HASH: [u8; 32] = [1u8; 32];
```

**Key Points:**
- Single function calls use `execute_on_session`
- Capability checking is automatic
- State updates happen automatically

## Step 4: Bundle Operations

For complex operations, use bundles:

```rust
pub fn close_voting_and_tally(
    ctx: Context<CloseVotingAndTally>,
) -> Result<()> {
    // Bundle multiple related operations
    let operations = vec![
        // 1. Stop accepting new votes
        SimpleOperation {
            function_hash: CLOSE_VOTING_FUNCTION_HASH,
            required_capabilities: Capability::Write.to_mask(),
            args: vec![], // No args needed
        },
        
        // 2. Calculate final results
        SimpleOperation {
            function_hash: TALLY_VOTES_FUNCTION_HASH,
            required_capabilities: Capability::Read.to_mask() | Capability::Write.to_mask(),
            args: vec![], // Reads current state, updates with results
        },
        
        // 3. Publish results
        SimpleOperation {
            function_hash: PUBLISH_RESULTS_FUNCTION_HASH,
            required_capabilities: Capability::Write.to_mask(),
            args: anchor_lang::AnchorSerialize::try_to_vec(&PublishArgs {
                timestamp: Clock::get()?.unix_timestamp,
            })?,
        },
    ];

    let bundle = SimpleBundle {
        session: ctx.accounts.session.key(),
        operations,
    };

    // Execute all operations atomically
    execute_bundle_v2(ctx.accounts.bundle_ctx, bundle)?;
    
    msg!("Voting closed and results tallied");
    Ok(())
}

const CLOSE_VOTING_FUNCTION_HASH: [u8; 32] = [2u8; 32];
const TALLY_VOTES_FUNCTION_HASH: [u8; 32] = [3u8; 32];
const PUBLISH_RESULTS_FUNCTION_HASH: [u8; 32] = [4u8; 32];

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct PublishArgs {
    pub timestamp: i64,
}
```

**Key Points:**
- Bundles execute multiple operations atomically
- Each operation can have different capability requirements
- All operations succeed or all fail

## Step 5: Error Handling

Handle capability errors gracefully:

```rust
pub fn admin_reset_voting(
    ctx: Context<AdminResetVoting>,
) -> Result<()> {
    // Try admin operation
    let reset_data = ResetData {
        admin: ctx.accounts.admin.key(),
        reason: "Manual reset requested".to_string(),
    };
    
    let args = anchor_lang::AnchorSerialize::try_to_vec(&reset_data)?;
    
    match execute_on_session(ctx.accounts.session_ctx, RESET_FUNCTION_HASH, args) {
        Ok(_) => {
            msg!("Voting session reset by admin");
            Ok(())
        },
        Err(e) => {
            match e {
                // Handle specific error types
                anchor_lang::error::Error::AnchorError(ae) if ae.error_code_number == 6005 => {
                    // InsufficientCapabilities error code
                    msg!("Error: Admin capabilities required for reset");
                    Err(VotingError::NotAuthorized.into())
                },
                _ => {
                    msg!("Error: Reset failed: {:?}", e);
                    Err(e)
                }
            }
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ResetData {
    pub admin: Pubkey,
    pub reason: String,
}

const RESET_FUNCTION_HASH: [u8; 32] = [5u8; 32];

#[error_code]
pub enum VotingError {
    #[msg("Not authorized for this operation")]
    NotAuthorized,
}
```

## Step 6: Complete Account Contexts

Here are the clean account contexts (notice how simple they are):

```rust
#[derive(Accounts)]
pub struct CreateVotingSession<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub session_ctx: CreateSessionV2<'info>,
}

#[derive(Accounts)]
pub struct CastVote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,
    pub session_ctx: ExecuteOnSession<'info>,
}

#[derive(Accounts)]
pub struct CloseVotingAndTally<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub session: Account<'info, Session>,
    pub bundle_ctx: ExecuteBundleV2<'info>,
}

#[derive(Accounts)]
pub struct AdminResetVoting<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    pub session_ctx: ExecuteOnSession<'info>,
}
```

## Step 7: Testing Your Application

Create tests to verify your voting system:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::*;
    use solana_program_test::*;

    #[tokio::test]
    async fn test_voting_workflow() {
        // Setup test environment
        let mut pt = ProgramTest::new("voting_app", voting_app::ID, processor!(voting_app::entry));
        let (mut banks_client, payer, recent_blockhash) = pt.start().await;

        // Test 1: Create voting session
        // ... test creation

        // Test 2: Cast votes
        // ... test voting

        // Test 3: Close and tally
        // ... test bundle execution

        println!("✅ All voting tests passed!");
    }
}
```

## Advanced Patterns

### Pattern 1: Dynamic Capability Requirements

```rust
pub fn conditional_operation(
    ctx: Context<ConditionalOperation>,
    operation_type: OperationType,
) -> Result<()> {
    let (function_hash, required_caps) = match operation_type {
        OperationType::ReadOnly => (READ_FUNCTION_HASH, Capability::Read.to_mask()),
        OperationType::WriteData => (WRITE_FUNCTION_HASH, Capability::Write.to_mask()),
        OperationType::AdminAction => (ADMIN_FUNCTION_HASH, Capability::Admin.to_mask()),
    };

    let args = anchor_lang::AnchorSerialize::try_to_vec(&operation_type)?;
    execute_on_session(ctx.accounts.session_ctx, function_hash, args)?;
    
    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum OperationType {
    ReadOnly,
    WriteData, 
    AdminAction,
}
```

### Pattern 2: Session Factory

```rust
pub fn create_specialized_session(
    ctx: Context<CreateSpecializedSession>,
    session_type: SessionType,
) -> Result<Pubkey> {
    let (capabilities, namespace) = match session_type {
        SessionType::Voter => (
            Capability::Read.to_mask() | Capability::Write.to_mask(),
            "voter-session"
        ),
        SessionType::Admin => (
            Capability::Read.to_mask() | Capability::Write.to_mask() | Capability::Admin.to_mask(),
            "admin-session"
        ),
        SessionType::ReadOnly => (
            Capability::Read.to_mask(),
            "readonly-session"
        ),
    };

    let session = create_session_v2(
        ctx.accounts.session_ctx,
        capabilities,
        vec![], // Empty initial state
        namespace.to_string(),
        1,
        vec![]
    )?;

    Ok(session)
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum SessionType {
    Voter,
    Admin,
    ReadOnly,
}
```

## Common Pitfalls and Solutions

### Pitfall 1: Forgetting Required Capabilities

```rust
// ❌ Wrong: Will fail at runtime
SimpleOperation {
    function_hash: WRITE_FUNCTION_HASH,
    required_capabilities: Capability::Read.to_mask(), // Wrong! Needs Write
    args: data,
}

// ✅ Correct: Match function requirements
SimpleOperation {
    function_hash: WRITE_FUNCTION_HASH,
    required_capabilities: Capability::Write.to_mask(), // Correct
    args: data,
}
```

### Pitfall 2: Over-requesting Capabilities

```rust
// ❌ Wrong: Too many capabilities (security risk)
let capabilities = Capability::Admin.to_mask(); // Don't need admin for voting

// ✅ Correct: Minimal required capabilities
let capabilities = Capability::Read.to_mask() | Capability::Write.to_mask();
```

### Pitfall 3: Ignoring Bundle Atomicity

```rust
// ❌ Wrong: Separate operations (not atomic)
execute_on_session(ctx, OPERATION_1_HASH, args1)?;
execute_on_session(ctx, OPERATION_2_HASH, args2)?; // If this fails, operation 1 already happened

// ✅ Correct: Bundle for atomicity
let bundle = SimpleBundle {
    session: ctx.session,
    operations: vec![
        SimpleOperation { function_hash: OPERATION_1_HASH, required_capabilities: caps1, args: args1 },
        SimpleOperation { function_hash: OPERATION_2_HASH, required_capabilities: caps2, args: args2 },
    ],
};
execute_bundle_v2(ctx, bundle)?; // All succeed or all fail
```

## Next Steps

Congratulations! You now understand Session V2. Here's what to explore next:

1. **Build a real application** - Try the token swap example
2. **Optimize performance** - Use capability bitmasks efficiently  
3. **Handle errors gracefully** - Implement retry logic
4. **Use advanced patterns** - Dynamic bundles, session factories

## Key Takeaways

- **Sessions are your only abstraction** - no account complexity
- **Capabilities are checked automatically** - just specify what you need
- **Operations update state automatically** - no manual synchronization
- **Bundles provide atomicity** - all operations succeed or fail together
- **Performance is optimized** - O(1) capability checking, direct execution

You're now ready to build fast, secure, and simple applications with Session V2! 🎉

## Resources

- [Session V2 API Reference](session-v2-api.md)
- [Token Swap Example](../examples/token_swap_v2/)
- [Performance Benchmarks](../tests/session_v2/performance_benchmarks.rs)
- [Community Forum](https://forum.valence.network)

Happy building! 🚀 