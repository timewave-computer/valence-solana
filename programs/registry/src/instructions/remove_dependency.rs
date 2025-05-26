use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::RegistryError;

/// Remove a dependency from a library
pub fn handler(
    ctx: Context<RemoveDependency>,
    dependency_program_id: Pubkey,
) -> Result<()> {
    let library_info = &mut ctx.accounts.library_info;
    
    // Find and remove the dependency
    let initial_len = library_info.dependencies.len();
    library_info.dependencies.retain(|dep| dep.program_id != dependency_program_id);
    
    // Check if dependency was found and removed
    if library_info.dependencies.len() == initial_len {
        return Err(RegistryError::DependencyNotFound.into());
    }
    
    library_info.last_updated = Clock::get()?.unix_timestamp;
    
    msg!("Removed dependency {} from library {}", 
         dependency_program_id, 
         library_info.program_id);
    
    Ok(())
} 