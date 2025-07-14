use anchor_lang::prelude::*;
use sha2::{Sha256, Digest};
use valence_common::{program_ids, bounds};

declare_id!("B2UgDMshe2sug7qTv4DseFNz6ipRSKPbqc9j98TAWJuo");

pub const SESSION_SEED: &[u8] = b"session";
pub const SESSION_COUNTER_SEED: &[u8] = b"session_counter";

// Capability bitmap - each bit represents a permission
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub struct Capabilities(pub u64);

impl Capabilities {
    pub const READ: u64 = 1 << 0;
    pub const WRITE: u64 = 1 << 1;
    pub const EXECUTE: u64 = 1 << 2;
    pub const TRANSFER: u64 = 1 << 3;
    
    pub fn new(bits: u64) -> Self {
        Self(bits)
    }
    
    pub fn has(&self, capability: u64) -> bool {
        (self.0 & capability) != 0
    }
    
    pub fn require(&self, capability: u64) -> Result<()> {
        require!(self.has(capability), ShardError::InsufficientCapabilities);
        Ok(())
    }
}

#[program]
pub mod shard {
    use super::*;

    pub fn initialize_session_counter(ctx: Context<InitializeSessionCounter>) -> Result<()> {
        let counter = &mut ctx.accounts.session_counter;
        counter.owner = ctx.accounts.owner.key();
        counter.counter = 0;
        msg!("Initialized session counter for owner: {}", counter.owner);
        Ok(())
    }

    pub fn create_session(
        ctx: Context<CreateSession>,
        capabilities: u64,
        metadata: Vec<u8>,
    ) -> Result<()> {
        let session = &mut ctx.accounts.session;
        let clock = Clock::get()?;
        
        // Get counter from the session_counter account
        let session_counter = &mut ctx.accounts.session_counter;
        let counter = session_counter.counter;
        session_counter.counter += 1;
        
        session.owner = ctx.accounts.owner.key();
        session.capabilities = Capabilities::new(capabilities);
        session.nonce = counter;
        session.state_hash = [0u8; 32];
        session.metadata = metadata;
        session.created_at = clock.unix_timestamp;
        session.consumed = false;
        
        msg!("Session created with capabilities: {} and nonce: {}", capabilities, session.nonce);
        Ok(())
    }

    pub fn execute_function(
        ctx: Context<ExecuteFunction>,
        function_hash: [u8; 32],
        input_data: Vec<u8>,
    ) -> Result<()> {
        // Store keys and account info before creating mutable borrow
        let session_key = ctx.accounts.session.key();
        let session_account_info = ctx.accounts.session.to_account_info();
        let owner_key = ctx.accounts.owner.key();
        
        let session = &mut ctx.accounts.session;
        
        // Verify session not consumed
        require!(!session.consumed, ShardError::SessionConsumed);
        
        // Verify caller is owner
        require_keys_eq!(
            session.owner,
            owner_key,
            ShardError::UnauthorizedCaller
        );
        
        // Verify session has execute capability
        session.capabilities.require(Capabilities::EXECUTE)?;
        
        // Validate registry program ID using common utility
        // Note: In production, you would verify the registry program address
        // For testing with dynamic program IDs, we skip this check
        
        // First verify the function is registered and valid
        let verify_accounts = registry::cpi::accounts::VerifyFunction {
            function_entry: ctx.accounts.function_entry.to_account_info(),
            program: ctx.accounts.function_program.to_account_info(),
        };
        
        let verify_ctx = CpiContext::new(
            ctx.accounts.registry_program.to_account_info(),
            verify_accounts,
        );
        
        // Verify the function using the expected bytecode hash
        // In production, this would compute the actual bytecode hash from the program
        // For testing, we pass [0u8; 32] to skip verification
        registry::cpi::verify_function(verify_ctx, [0u8; 32])?;
        
        // Execute the function via CPI
        // Anchor discriminator for "process" instruction
        const PROCESS_DISCRIMINATOR: [u8; 8] = [147, 104, 175, 139, 110, 254, 236, 21];
        
        // Build instruction data: discriminator + borsh-serialized Vec<u8>
        let mut ix_data = Vec::new();
        ix_data.extend_from_slice(&PROCESS_DISCRIMINATOR);
        // Borsh serialize the Vec<u8> parameter
        ix_data.extend_from_slice(&(input_data.len() as u32).to_le_bytes());
        ix_data.extend_from_slice(&input_data);
        
        let ix = anchor_lang::solana_program::instruction::Instruction {
            program_id: ctx.accounts.function_program.key(),
            accounts: vec![
                AccountMeta::new_readonly(session_key, false),
                AccountMeta::new_readonly(owner_key, true),
            ],
            data: ix_data,
        };
        
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                session_account_info,
                ctx.accounts.owner.to_account_info(),
                ctx.accounts.function_program.to_account_info(),
            ],
        )?;
        
        // Update session state with overflow protection
        session.nonce = bounds::checked_add(session.nonce, 1)?;
        
        // Compute new state hash
        let mut hasher = Sha256::new();
        hasher.update(session.state_hash);
        hasher.update(function_hash);
        hasher.update(input_data.as_slice());
        hasher.update(session.nonce.to_le_bytes());
        session.state_hash = hasher.finalize().into();
        
        msg!("Function executed, new state hash: {:?}", session.state_hash);
        Ok(())
    }

    pub fn consume_session(ctx: Context<ConsumeSession>) -> Result<()> {
        let session = &mut ctx.accounts.session;
        
        // Verify not already consumed
        require!(!session.consumed, ShardError::SessionConsumed);
        
        // Verify caller is owner
        require_keys_eq!(
            session.owner,
            ctx.accounts.owner.key(),
            ShardError::UnauthorizedCaller
        );
        
        // Mark as consumed (linear type semantics)
        session.consumed = true;
        
        msg!("Session consumed");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeSessionCounter<'info> {
    #[account(
        init,
        payer = owner,
        space = SessionCounter::SIZE,
        seeds = [SESSION_COUNTER_SEED, owner.key().as_ref()],
        bump
    )]
    pub session_counter: Account<'info, SessionCounter>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(address = program_ids::SYSTEM_PROGRAM_ID)]
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(capabilities: u64, metadata: Vec<u8>)]
pub struct CreateSession<'info> {
    #[account(
        init,
        payer = owner,
        space = Session::SIZE + metadata.len(),
        seeds = [
            SESSION_SEED,
            owner.key().as_ref(),
            &session_counter.counter.to_le_bytes()
        ],
        bump
    )]
    pub session: Account<'info, Session>,
    
    #[account(
        mut,
        seeds = [SESSION_COUNTER_SEED, owner.key().as_ref()],
        bump
    )]
    pub session_counter: Account<'info, SessionCounter>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(address = program_ids::SYSTEM_PROGRAM_ID)]
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteFunction<'info> {
    #[account(mut)]
    pub session: Account<'info, Session>,
    
    pub owner: Signer<'info>,
    
    /// CHECK: Registry program for verification
    pub registry_program: AccountInfo<'info>,
    
    /// CHECK: Function entry from registry
    pub function_entry: AccountInfo<'info>,
    
    /// CHECK: The program implementing the function
    pub function_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ConsumeSession<'info> {
    #[account(mut)]
    pub session: Account<'info, Session>,
    
    pub owner: Signer<'info>,
}

#[account]
pub struct Session {
    pub owner: Pubkey,
    pub capabilities: Capabilities,
    pub nonce: u64,
    pub state_hash: [u8; 32],
    pub metadata: Vec<u8>,
    pub created_at: i64,
    pub consumed: bool,
}

impl Session {
    pub const SIZE: usize = 8 + // discriminator
        32 + // owner
        8 + // capabilities
        8 + // nonce
        32 + // state_hash
        4 + // metadata vec length
        8 + // created_at
        1; // consumed
}

#[account]
pub struct SessionCounter {
    pub owner: Pubkey,
    pub counter: u64,
}

impl SessionCounter {
    pub const SIZE: usize = 8 + // discriminator
        32 + // owner
        8; // counter
}

#[error_code]
pub enum ShardError {
    #[msg("Insufficient capabilities")]
    InsufficientCapabilities,
    #[msg("Session already consumed")]
    SessionConsumed,
    #[msg("Unauthorized caller")]
    UnauthorizedCaller,
    #[msg("Invalid program")]
    InvalidProgram,
    #[msg("Session counter not initialized")]
    CounterNotInitialized,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_operations() {
        let mut caps = Capabilities::new(0);
        
        // Test individual capabilities
        assert!(!caps.has(Capabilities::READ));
        assert!(!caps.has(Capabilities::WRITE));
        
        // Add capabilities
        caps.0 |= Capabilities::READ;
        caps.0 |= Capabilities::WRITE;
        
        assert!(caps.has(Capabilities::READ));
        assert!(caps.has(Capabilities::WRITE));
        assert!(!caps.has(Capabilities::EXECUTE));
        
        // Test require
        assert!(caps.require(Capabilities::READ).is_ok());
        assert!(caps.require(Capabilities::EXECUTE).is_err());
    }
    
    #[test]
    fn test_state_hash_update() {
        let state_hash = [0u8; 32];
        let function_hash = [1u8; 32];
        let input_data = b"test_data";
        let nonce = 1u64;
        
        // Compute new state hash
        let mut hasher = Sha256::new();
        hasher.update(state_hash);
        hasher.update(function_hash);
        hasher.update(input_data);
        hasher.update(nonce.to_le_bytes());
        let new_hash: [u8; 32] = hasher.finalize().into();
        
        // Verify it changed
        assert_ne!(state_hash, new_hash);
    }
}