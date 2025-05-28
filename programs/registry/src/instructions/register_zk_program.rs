// Register ZK program instruction handler for Registry Program

use anchor_lang::prelude::*;
use crate::state::*;

pub fn handler(
    ctx: Context<RegisterZKProgram>,
    program_id: Pubkey,
    verification_key_hash: [u8; 32],
    program_type: String,
    description: String,
) -> Result<()> {
    let zk_program_info = &mut ctx.accounts.zk_program_info;
    
    // Initialize ZK program info
    zk_program_info.program_id = program_id;
    zk_program_info.verification_key_hash = verification_key_hash;
    zk_program_info.program_type = program_type.clone();
    zk_program_info.description = description.clone();
    zk_program_info.is_active = true;
    zk_program_info.registered_at = Clock::get()?.unix_timestamp;
    zk_program_info.last_verified = 0;
    zk_program_info.verification_count = 0;
    zk_program_info.bump = ctx.bumps.zk_program_info;
    
    msg!("Registered ZK program {} of type '{}' with verification key hash: {:?}", 
         program_id, program_type, verification_key_hash);
    
    Ok(())
} 