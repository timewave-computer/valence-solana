//! Registry instructions - Core logic (controllers)

use anchor_lang::prelude::*;
use crate::{FunctionEntry, RegistryError};

/// Register a new function
pub fn register(
    ctx: Context<Register>,
    hash: [u8; 32],
    program: Pubkey,
    required_capabilities: Vec<String>,
) -> Result<()> {
    // Validate inputs
    validate_hash(&hash)?;
    validate_program(&program)?;
    validate_capabilities(&required_capabilities)?;

    // Initialize function entry
    let function_entry = &mut ctx.accounts.function_entry;
    function_entry.hash = hash;
    function_entry.program = program;
    function_entry.authority = ctx.accounts.authority.key();
    function_entry.required_capabilities = required_capabilities;

    msg!("Registered function with hash {:?} at {}", hash, program);
    Ok(())
}

/// Unregister a function
pub fn unregister(ctx: Context<Unregister>, hash: [u8; 32]) -> Result<()> {
    let function_entry = &ctx.accounts.function_entry;
    
    // Verify hash matches
    require!(
        function_entry.hash == hash,
        RegistryError::HashMismatch
    );
    
    // Verify authority
    require!(
        function_entry.authority == ctx.accounts.authority.key(),
        RegistryError::Unauthorized
    );

    msg!("Unregistered function with hash {:?}", function_entry.hash);
    
    // Account will be closed by Anchor's close constraint
    Ok(())
}

/// Validate function hash
pub fn validate_hash(hash: &[u8; 32]) -> Result<()> {
    // Check hash is not all zeros
    require!(
        hash.iter().any(|&b| b != 0),
        RegistryError::InvalidHash
    );
    Ok(())
}

/// Validate program ID
pub fn validate_program(program: &Pubkey) -> Result<()> {
    require!(
        *program != Pubkey::default(),
        RegistryError::InvalidProgram
    );
    Ok(())
}

/// Validate capabilities
pub fn validate_capabilities(capabilities: &[String]) -> Result<()> {
    // Check for duplicates
    let mut seen = std::collections::HashSet::new();
    for cap in capabilities {
        // Validate capability format (non-empty, reasonable length)
        require!(
            !cap.is_empty() && cap.len() <= 64,
            RegistryError::InvalidCapability
        );
        
        // Check for duplicates
        require!(
            seen.insert(cap),
            RegistryError::DuplicateCapability
        );
    }
    Ok(())
}

// Account contexts

#[derive(Accounts)]
#[instruction(hash: [u8; 32])]
pub struct Register<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 32 + 4 + (10 * 64), // discriminator + hash + program + authority + vec length + max 10 capabilities of 64 chars each
        seeds = [b"function", hash.as_ref()],
        bump,
    )]
    pub function_entry: Account<'info, FunctionEntry>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(hash: [u8; 32])]
pub struct Unregister<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"function", hash.as_ref()],
        bump,
        close = authority,
    )]
    pub function_entry: Account<'info, FunctionEntry>,
}

