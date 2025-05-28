// Remove registry instruction for ZK Verifier Program

use anchor_lang::prelude::*;


pub fn handler(
    ctx: Context<crate::RemoveRegistry>,
    registry_id: u64,
) -> Result<()> {
    let verifier_state = &mut ctx.accounts.verifier_state;
    
    // Decrement total keys counter
    verifier_state.total_keys = verifier_state.total_keys.checked_sub(1)
        .ok_or(error!(crate::error::VerifierError::ArithmeticUnderflow))?;
    
    msg!("Removed verification key for program: {}, registry: {}", 
         ctx.accounts.owner.key(), registry_id);
    
    Ok(())
}



 