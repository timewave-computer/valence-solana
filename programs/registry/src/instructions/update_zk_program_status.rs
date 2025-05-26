// Update ZK program status instruction handler for Registry Program

use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::RegistryError;

pub fn handler(
    ctx: Context<UpdateZKProgramStatus>,
    is_active: bool,
) -> Result<()> {
    let zk_program_info = &mut ctx.accounts.zk_program_info;
    
    // Update the active status
    zk_program_info.is_active = is_active;
    
    msg!("Updated ZK program {} status to active: {}", 
         zk_program_info.program_id, is_active);
    
    Ok(())
} 