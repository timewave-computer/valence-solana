use anchor_lang::prelude::*;
use crate::state::AuthorizationState;
use crate::error::AuthorizationError;

pub fn handler(ctx: Context<LookupAuthorization>, label: String) -> Result<Pubkey> {
    // Validate label length
    if label.is_empty() || label.len() > 32 {
        return Err(AuthorizationError::InvalidParameters.into());
    }
    
    // Compute the authorization PDA
    let (auth_pda, _) = Pubkey::find_program_address(
        &[b"authorization".as_ref(), label.as_bytes()],
        ctx.program_id
    );
    
    msg!("Found authorization at: {}", auth_pda);
    
    Ok(auth_pda)
}

#[derive(Accounts)]
#[instruction(label: String)]
pub struct LookupAuthorization<'info> {
    #[account(
        seeds = [b"authorization_state".as_ref()],
        bump = authorization_state.bump,
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
} 