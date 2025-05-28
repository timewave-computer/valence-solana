use anchor_lang::prelude::*;
use crate::state::*;

/// Check if a library's dependencies are compatible with their current versions
pub fn handler(
    ctx: Context<CheckVersionCompatibility>,
) -> Result<bool> {
    let library_info = &ctx.accounts.library_info;
    
    // Check each dependency for version compatibility
    for dependency in &library_info.dependencies {
        // For now, we'll do a simple check - in a full implementation,
        // this would query the actual dependency library accounts
        if !is_version_compatible(&dependency.required_version, "1.0.0") {
            msg!("Version incompatibility detected for dependency {}: required {}, found 1.0.0", 
                 dependency.program_id, 
                 dependency.required_version);
            return Ok(false);
        }
    }
    
    msg!("All dependencies are version compatible for library {}", 
         library_info.program_id);
    
    Ok(true)
}

/// Check if an available version satisfies a required version constraint
/// This is a simplified implementation - a full version would support ranges like "^1.0.0", "~1.2.0", etc.
fn is_version_compatible(required: &str, available: &str) -> bool {
    let req_parts: Vec<u32> = required.split('.').filter_map(|s| s.parse().ok()).collect();
    let avail_parts: Vec<u32> = available.split('.').filter_map(|s| s.parse().ok()).collect();
    
    if req_parts.len() != 3 || avail_parts.len() != 3 {
        return false;
    }
    
    // Simple exact match for now - could be extended to support semantic versioning ranges
    req_parts == avail_parts
} 