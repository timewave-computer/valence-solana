use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::SessionFactoryError;

#[derive(Accounts)]
pub struct CreateSession<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"factory_state"],
        bump = factory_state.bump,
    )]
    pub factory_state: Account<'info, FactoryState>,
    
    #[account(
        init,
        payer = owner,
        space = Session::get_space(5), // Support up to 5 namespaces initially
        seeds = [
            b"session",
            owner.key().as_ref(),
            factory_state.total_sessions_created.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub session: Account<'info, Session>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateSession>,
    eval_program_id: Pubkey,
    initial_namespaces: Vec<[u8; 32]>,
) -> Result<()> {
    let factory_state = &mut ctx.accounts.factory_state;
    let session = &mut ctx.accounts.session;
    let clock = Clock::get()?;
    
    // Validate namespace count
    require!(
        initial_namespaces.len() <= 5,
        SessionFactoryError::TooManyNamespaces
    );
    
    // Initialize session
    session.owner = ctx.accounts.owner.key();
    session.eval_program_id = eval_program_id;
    session.namespaces = initial_namespaces;
    session.nonce = 0;
    session.created_at = clock.unix_timestamp;
    session.last_activity = clock.unix_timestamp;
    session.is_active = true;
    session.bump = ctx.bumps.session;
    
    // Update factory state
    factory_state.total_sessions_created += 1;
    
    msg!(
        "Session created for owner: {} with eval: {}",
        session.owner,
        session.eval_program_id
    );
    msg!("Session address: {}", ctx.accounts.session.key());
    msg!("Namespaces count: {}", session.namespaces.len());
    
    Ok(())
} 