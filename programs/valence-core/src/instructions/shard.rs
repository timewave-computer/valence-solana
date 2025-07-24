use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak;
use crate::{state::*, errors::*, instructions::{verify_account, VerifyAccount}};

// ================================
// Shard Management Instructions
// ================================

// ===== Shard Deployment =====

/// Deploy a new shard with code/state definition
pub fn deploy(ctx: Context<Deploy>, code: Vec<u8>) -> Result<()> {
    // ===== Code Size Validation =====
    
    // Ensure code doesn't exceed maximum allowed size
    require!(code.len() <= MAX_CODE_SIZE, CoreError::CodeTooLarge);
    
    // ===== Initialize Shard State =====
    
    let shard = &mut ctx.accounts.shard;
    
    // Set PDA bump for future derivation
    shard.bump = ctx.bumps.shard;
    
    // Compute and store code hash for integrity verification
    shard.code_hash = keccak::hash(&code).to_bytes();
    
    // Set deployer as shard owner
    shard.owner = ctx.accounts.owner.key();
    
    // ===== Store Code Separately =====
    
    // Store actual code in dedicated account for size flexibility
    let code_account = &mut ctx.accounts.code;
    code_account.set_inner(CodeStorage {
        code: code.clone(),
    });
    
    // Log deployment details for monitoring
    msg!("Shard deployed with {} bytes of code, hash: {:?}", 
        code.len(), 
        &shard.code_hash[..8]
    );
    
    Ok(())
}

// ===== Shard Execution =====

/// Execute shard with session account authorization
pub fn execute(ctx: Context<Execute>, input: Vec<u8>) -> Result<()> {
    let shard = &ctx.accounts.shard;
    let account = &ctx.accounts.account;
    
    // ===== Code Integrity Verification =====
    
    // Load stored code and compute its hash
    let code_account = &ctx.accounts.code;
    let stored_code = &code_account.code;
    let computed_hash = keccak::hash(stored_code).to_bytes();
    
    // Ensure code hasn't been tampered with
    require!(
        computed_hash == shard.code_hash,
        CoreError::CodeHashMismatch
    );
    
    // ===== Account Validation =====
    
    // Check account hasn't expired or exceeded usage limits
    require!(account.is_active(), CoreError::AccountExpired);
    
    // ===== Authorization via CPI =====
    
    // Prepare CPI context for verifier authorization
    let cpi_accounts = VerifyAccount {
        account: ctx.accounts.account.to_account_info(),
        caller: ctx.accounts.caller.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(
        ctx.accounts.verifier_program.to_account_info(),
        cpi_accounts,
    );
    
    // Delegate authorization to external verifier
    verify_account(cpi_ctx)?;
    
    // ===== Update Account State =====
    
    let account = &mut ctx.accounts.account;
    
    // Increment nonce for replay protection
    account.nonce = account.nonce.checked_add(1).ok_or(CoreError::NonceOverflow)?;
    
    // Track usage for lifecycle management
    account.uses = account.uses.checked_add(1).ok_or(CoreError::UsageOverflow)?;
    
    // ===== Execute Shard Logic =====
    
    // Shards store program IDs or DSL code that protocols interpret
    // The actual execution is protocol-specific and happens off-chain
    msg!("Executing shard {} with {} bytes of input", shard.key(), input.len());
    msg!("Code hash verified: {:?}", &shard.code_hash[..8]);
    msg!("Account {} has been used {} times", account.key(), account.uses);
    
    Ok(())
}

// ================================
// Account Validation Contexts
// ================================

// ===== Deployment Context =====

/// Validation context for deploying a new shard
#[derive(Accounts)]
pub struct Deploy<'info> {
    // Shard deployer who pays rent and owns the shard
    #[account(mut)]
    pub owner: Signer<'info>,
    
    // New shard account to initialize
    #[account(
        init,
        payer = owner,
        space = 8 + Shard::SIZE,
        seeds = [SHARD_SEED, shard_id.key().as_ref()],
        bump
    )]
    pub shard: Account<'info, Shard>,
    
    // Separate account for storing shard code
    #[account(
        init,
        payer = owner,
        space = 8 + 4 + MAX_CODE_SIZE, // discriminator + vec length + code
        seeds = [CODE_SEED, shard_id.key().as_ref()],
        bump
    )]
    pub code: Account<'info, CodeStorage>,
    
    // Unique identifier for shard PDA derivation
    /// CHECK: Random pubkey for shard ID
    pub shard_id: UncheckedAccount<'info>,
    
    // Required for account creation
    pub system_program: Program<'info, System>,
}

// ===== Execution Context =====

/// Validation context for executing shard logic
#[derive(Accounts)]
pub struct Execute<'info> {
    // Entity requesting shard execution
    pub caller: Signer<'info>,
    
    // Shard containing execution metadata
    #[account(
        seeds = [SHARD_SEED, shard.key().as_ref()],
        bump = shard.bump
    )]
    pub shard: Account<'info, Shard>,
    
    // Code storage for integrity verification
    #[account(
        seeds = [CODE_SEED, shard.key().as_ref()],
        bump
    )]
    pub code: Account<'info, CodeStorage>,
    
    // Session account used for authorization
    #[account(mut)]
    pub account: Account<'info, SessionAccount>,
    
    // External verifier that authorizes execution
    /// CHECK: Verifier program
    pub verifier_program: UncheckedAccount<'info>,
}