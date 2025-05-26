use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::RegistryError;

pub fn handler(
    ctx: Context<RegisterLibrary>,
    library_type: String,
    description: String,
    is_approved: bool,
) -> Result<()> {
    // Validate input parameters
    if library_type.len() > 50 {
        return Err(error!(RegistryError::LibraryTypeTooLong));
    }
    
    if description.len() > 200 {
        return Err(error!(RegistryError::DescriptionTooLong));
    }
    
    // Get the library info account
    let library_info = &mut ctx.accounts.library_info;
    
    // Set the program ID
    library_info.program_id = ctx.accounts.program_id.key();
    
    // Set the library type
    library_info.library_type = library_type;
    
    // Set the description
    library_info.description = description;
    
    // Set approved status (only owner can set this to true initially)
    library_info.is_approved = is_approved;
    
    // Set version to initial value
    library_info.version = "1.0.0".to_string();
    
    // Set last updated timestamp
    library_info.last_updated = Clock::get()?.unix_timestamp;
    
    // Initialize empty dependencies
    library_info.dependencies = Vec::new();
    
    // Store the bump seed
    library_info.bump = ctx.bumps.library_info;
    
    // Log the registration
    msg!(
        "Library registered: Program ID: {}, Type: {}, Approved: {}",
        library_info.program_id,
        library_info.library_type,
        library_info.is_approved
    );
    
    Ok(())
} 