// Verify ZK program registration instruction handler for Registry Program

use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::RegistryError;

pub fn handler(
    ctx: Context<VerifyZKProgram>,
    program_id: Pubkey,
) -> Result<bool> {
    let zk_program_info = &mut ctx.accounts.zk_program_info;
    
    // Verify the program ID matches
    if zk_program_info.program_id != program_id {
        return Ok(false);
    }
    
    // Check if the program is active
    if !zk_program_info.is_active {
        return Ok(false);
    }
    
    // Update verification statistics
    zk_program_info.last_verified = Clock::get()?.unix_timestamp;
    zk_program_info.verification_count = zk_program_info.verification_count
        .checked_add(1)
        .ok_or(RegistryError::ArithmeticOverflow)?;
    
    msg!("Verified ZK program {} (verification count: {})", 
         program_id, zk_program_info.verification_count);
    
    Ok(true)
} 