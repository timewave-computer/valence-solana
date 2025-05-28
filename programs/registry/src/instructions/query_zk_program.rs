// Query ZK program instruction handler for Registry Program

use anchor_lang::prelude::*;
use crate::state::*;

pub fn handler(
    ctx: Context<QueryZKProgram>,
) -> Result<ZKProgramInfo> {
    let zk_program_info = &ctx.accounts.zk_program_info;
    
    msg!("Queried ZK program {} of type '{}', active: {}", 
         zk_program_info.program_id, 
         zk_program_info.program_type, 
         zk_program_info.is_active);
    
    // Return a copy of the ZK program info
    Ok(ZKProgramInfo {
        program_id: zk_program_info.program_id,
        verification_key_hash: zk_program_info.verification_key_hash,
        program_type: zk_program_info.program_type.clone(),
        description: zk_program_info.description.clone(),
        is_active: zk_program_info.is_active,
        registered_at: zk_program_info.registered_at,
        last_verified: zk_program_info.last_verified,
        verification_count: zk_program_info.verification_count,
        bump: zk_program_info.bump,
    })
} 