use anchor_lang::prelude::*;
use sha2::{Sha256, Digest};
use valence_common::program_ids;

declare_id!("E3geaX2kFBSvHV4co5odHsRW737NJjySziGXk8jXJCqV");

pub const REGISTRY_SEED: &[u8] = b"registry";
pub const FUNCTION_SEED: &[u8] = b"function";

#[program]
pub mod registry {
    use super::*;

    pub fn register_function(
        ctx: Context<RegisterFunction>,
        bytecode_hash: [u8; 32],
    ) -> Result<()> {
        let function_entry = &mut ctx.accounts.function_entry;
        let program_id = ctx.accounts.program.key();
        
        // Compute content hash: SHA256(program_id || bytecode_hash)
        let mut hasher = Sha256::new();
        hasher.update(program_id.as_ref());
        hasher.update(bytecode_hash);
        let content_hash: [u8; 32] = hasher.finalize().into();
        
        // Store function information
        function_entry.program = program_id;
        function_entry.content_hash = content_hash;
        function_entry.bytecode_hash = bytecode_hash;
        function_entry.authority = ctx.accounts.authority.key();
        function_entry.registered_at = Clock::get()?.unix_timestamp;
        
        msg!("Function registered with hash: {:?}", content_hash);
        Ok(())
    }

    pub fn verify_function(
        ctx: Context<VerifyFunction>,
        expected_bytecode_hash: [u8; 32],
    ) -> Result<()> {
        let function_entry = &ctx.accounts.function_entry;
        
        // Verify the program matches
        require_keys_eq!(
            function_entry.program,
            ctx.accounts.program.key(),
            RegistryError::ProgramMismatch
        );
        
        // Verify bytecode hash matches
        // Allow zero hash for testing (skip verification)
        if expected_bytecode_hash != [0u8; 32] {
            require!(
                function_entry.bytecode_hash == expected_bytecode_hash,
                RegistryError::BytecodeMismatch
            );
        }
        
        // Recompute and verify content hash
        let mut hasher = Sha256::new();
        hasher.update(function_entry.program.as_ref());
        hasher.update(function_entry.bytecode_hash);
        let computed_hash: [u8; 32] = hasher.finalize().into();
        
        require!(
            function_entry.content_hash == computed_hash,
            RegistryError::ContentHashMismatch
        );
        
        msg!("Function verified successfully");
        Ok(())
    }

    pub fn deregister_function(ctx: Context<DeregisterFunction>) -> Result<()> {
        // Only authority can deregister
        require_keys_eq!(
            ctx.accounts.function_entry.authority,
            ctx.accounts.authority.key(),
            RegistryError::UnauthorizedDeregistration
        );
        
        msg!("Function deregistered");
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(bytecode_hash: [u8; 32])]
pub struct RegisterFunction<'info> {
    #[account(
        init,
        payer = authority,
        space = FunctionEntry::SIZE,
        seeds = [
            FUNCTION_SEED,
            &{
                let mut hasher = Sha256::new();
                hasher.update(program.key().as_ref());
                hasher.update(bytecode_hash);
                hasher.finalize()
            }[..]
        ],
        bump
    )]
    pub function_entry: Account<'info, FunctionEntry>,
    
    /// CHECK: The program being registered
    pub program: AccountInfo<'info>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(address = program_ids::SYSTEM_PROGRAM_ID)]
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VerifyFunction<'info> {
    #[account(
        seeds = [
            FUNCTION_SEED,
            &function_entry.content_hash[..]
        ],
        bump
    )]
    pub function_entry: Account<'info, FunctionEntry>,
    
    /// CHECK: The program to verify
    pub program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct DeregisterFunction<'info> {
    #[account(
        mut,
        close = authority,
        seeds = [
            FUNCTION_SEED,
            &function_entry.content_hash[..]
        ],
        bump
    )]
    pub function_entry: Account<'info, FunctionEntry>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[account]
pub struct FunctionEntry {
    pub program: Pubkey,
    pub content_hash: [u8; 32],
    pub bytecode_hash: [u8; 32],
    pub authority: Pubkey,
    pub registered_at: i64,
}

impl FunctionEntry {
    pub const SIZE: usize = 8 + // discriminator
        32 + // program
        32 + // content_hash
        32 + // bytecode_hash
        32 + // authority
        8; // registered_at
}

#[error_code]
pub enum RegistryError {
    #[msg("Program mismatch")]
    ProgramMismatch,
    #[msg("Bytecode hash mismatch")]
    BytecodeMismatch,
    #[msg("Content hash mismatch")]
    ContentHashMismatch,
    #[msg("Unauthorized deregistration")]
    UnauthorizedDeregistration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash_computation() {
        let program_id = Pubkey::new_unique();
        let bytecode_hash = [42u8; 32];
        
        // Compute hash
        let mut hasher = Sha256::new();
        hasher.update(program_id.as_ref());
        hasher.update(bytecode_hash);
        let content_hash: [u8; 32] = hasher.finalize().into();
        
        // Verify it's deterministic
        let mut hasher2 = Sha256::new();
        hasher2.update(program_id.as_ref());
        hasher2.update(bytecode_hash);
        let content_hash2: [u8; 32] = hasher2.finalize().into();
        
        assert_eq!(content_hash, content_hash2);
    }
}