// Manage CPI allowlist instruction
use crate::state::CpiAllowlistAccount;
use anchor_lang::prelude::*;

// ================================
// Instruction Handlers
// ================================

/// Initialize the CPI allowlist
pub fn initialize_allowlist(ctx: Context<InitializeAllowlist>) -> Result<()> {
    let allowlist = &mut ctx.accounts.cpi_allowlist;
    **allowlist = CpiAllowlistAccount::new(ctx.accounts.authority.key());
    Ok(())
}

/// Add a program to the allowlist
pub fn add_program_to_cpi_allowlist(
    ctx: Context<ManageAllowlist>, 
    program_id: Pubkey
) -> Result<()> {
    let allowlist = &mut ctx.accounts.cpi_allowlist;
    allowlist.add_program(program_id)?;
    Ok(())
}

/// Remove a program from the allowlist
pub fn remove_program_from_cpi_allowlist(
    ctx: Context<ManageAllowlist>,
    program_id: Pubkey
) -> Result<()> {
    let allowlist = &mut ctx.accounts.cpi_allowlist;
    allowlist.remove_program(&program_id)?;
    Ok(())
}

// ================================
// Account Contexts
// ================================

/// Initialize allowlist account context
#[derive(Accounts)]
pub struct InitializeAllowlist<'info> {
    #[account(
        init,
        payer = authority,
        space = CpiAllowlistAccount::space(),
        seeds = [b"cpi_allowlist"],
        bump
    )]
    pub cpi_allowlist: Account<'info, CpiAllowlistAccount>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Manage allowlist account context
#[derive(Accounts)]
pub struct ManageAllowlist<'info> {
    #[account(
        mut,
        seeds = [b"cpi_allowlist"],
        bump,
        has_one = authority
    )]
    pub cpi_allowlist: Account<'info, CpiAllowlistAccount>,
    
    pub authority: Signer<'info>,
}