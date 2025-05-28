// Initialize instruction for ZK Verifier Program

use anchor_lang::prelude::*;


pub fn handler(
    ctx: Context<crate::Initialize>,
    coprocessor_root: [u8; 32],
    verifier: Pubkey,
) -> Result<()> {
    let verifier_state = &mut ctx.accounts.verifier_state;
    
    verifier_state.owner = ctx.accounts.owner.key();
    verifier_state.coprocessor_root = coprocessor_root;
    verifier_state.verifier = verifier;
    verifier_state.total_keys = 0;
    verifier_state.bump = ctx.bumps.verifier_state;
    
    msg!("ZK Verifier initialized with coprocessor root: {:?}", coprocessor_root);
    
    Ok(())
}

 