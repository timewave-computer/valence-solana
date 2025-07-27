// Manage CPI allowlist instruction
use crate::state::CPIAllowlist;
use anchor_lang::prelude::*;

// ================================
// Instruction Handlers
// ================================

/// Initialize the CPI allowlist
pub fn initialize_allowlist(ctx: Context<InitializeAllowlist>) -> Result<()> {
    let allowlist = &mut ctx.accounts.cpi_allowlist;
    **allowlist = CPIAllowlist::new(ctx.accounts.authority.key());
    Ok(())
}

/// Add a program to the allowlist
pub fn add_to_allowlist(
    ctx: Context<ManageAllowlist>, 
    program_id: Pubkey
) -> Result<()> {
    let allowlist = &mut ctx.accounts.cpi_allowlist;
    allowlist.add_program(program_id)?;
    Ok(())
}

/// Remove a program from the allowlist
pub fn remove_from_allowlist(
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
        space = CPIAllowlist::space(),
        seeds = [b"cpi_allowlist"],
        bump
    )]
    pub cpi_allowlist: Account<'info, CPIAllowlist>,
    
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
    pub cpi_allowlist: Account<'info, CPIAllowlist>,
    
    pub authority: Signer<'info>,
}