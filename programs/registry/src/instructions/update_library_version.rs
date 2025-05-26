use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::RegistryError;

/// Update a library's version
pub fn handler(
    ctx: Context<UpdateLibraryVersion>,
    new_version: String,
) -> Result<()> {
    let library_info = &mut ctx.accounts.library_info;
    
    // Validate version string format (basic semantic versioning check)
    if !is_valid_version(&new_version) {
        return Err(RegistryError::InvalidVersionString.into());
    }
    
    // Check if new version is greater than current version
    if !is_version_greater(&new_version, &library_info.version) {
        return Err(RegistryError::InvalidDependencyVersion.into());
    }
    
    let old_version = library_info.version.clone();
    library_info.version = new_version.clone();
    library_info.last_updated = Clock::get()?.unix_timestamp;
    
    msg!("Updated library {} version from {} to {}", 
         library_info.program_id, 
         old_version, 
         new_version);
    
    Ok(())
}

/// Validate semantic version format (major.minor.patch)
pub fn is_valid_version(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    
    // Must have exactly 3 parts
    if parts.len() != 3 {
        return false;
    }
    
    // Each part must be a valid number
    for part in parts {
        if part.parse::<u32>().is_err() {
            return false;
        }
    }
    
    true
}

/// Check if version_a is greater than version_b (semantic versioning)
pub fn is_version_greater(version_a: &str, version_b: &str) -> bool {
    let parts_a: Vec<u32> = version_a.split('.').filter_map(|s| s.parse().ok()).collect();
    let parts_b: Vec<u32> = version_b.split('.').filter_map(|s| s.parse().ok()).collect();
    
    if parts_a.len() != 3 || parts_b.len() != 3 {
        return false;
    }
    
    // Compare major.minor.patch
    for i in 0..3 {
        if parts_a[i] > parts_b[i] {
            return true;
        } else if parts_a[i] < parts_b[i] {
            return false;
        }
    }
    
    false // versions are equal
} 