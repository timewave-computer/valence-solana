use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::RegistryError;

/// Add a dependency to a library
pub fn handler(
    ctx: Context<AddDependency>,
    dependency: LibraryDependency,
) -> Result<()> {
    let library_info = &mut ctx.accounts.library_info;
    let dependency_library = &ctx.accounts.dependency_library;
    
    // Validate that the dependency library exists and is approved
    if !dependency_library.is_approved {
        return Err(RegistryError::LibraryNotApproved.into());
    }
    
    // Check if dependency already exists
    for existing_dep in &library_info.dependencies {
        if existing_dep.program_id == dependency.program_id {
            return Err(RegistryError::DependencyAlreadyExists.into());
        }
    }
    
    // Add the dependency
    library_info.dependencies.push(dependency);
    library_info.last_updated = Clock::get()?.unix_timestamp;
    
    msg!("Added dependency {} to library {}", 
         dependency_library.program_id, 
         library_info.program_id);
    
    Ok(())
} 