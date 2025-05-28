use anchor_lang::prelude::*;
use crate::state::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct LibraryInfoResponse {
    pub program_id: Pubkey,
    pub library_type: String,
    pub description: String,
    pub is_approved: bool,
    pub version: String,
    pub last_updated: i64,
}

impl From<&LibraryInfo> for LibraryInfoResponse {
    fn from(info: &LibraryInfo) -> Self {
        Self {
            program_id: info.program_id,
            library_type: info.library_type.clone(),
            description: info.description.clone(),
            is_approved: info.is_approved,
            version: info.version.clone(),
            last_updated: info.last_updated,
        }
    }
}

pub fn handler(
    ctx: Context<ListLibraries>,
    start_after: Option<Pubkey>,
    limit: u8,
) -> Result<Vec<LibraryInfoResponse>> {
    // Cap the limit to prevent excessive computation
    let limit = limit.min(20);
    
    msg!("Listing libraries with start_after: {:?}, limit: {}", start_after, limit);
    
    // Get the registry program ID
    let program_id = crate::ID;
    
    // Calculate discriminator for LibraryInfo accounts
    let discriminator = LibraryInfo::DISCRIMINATOR;
    
    // Get all accounts of type LibraryInfo
    // To implement this in a production system, we would 
    // use the RPC getMultipleAccountsInfo to fetch accounts
    // or implement an in-program caching mechanism with pagination.
    
    // For this implementation, we'll use the account_info_iter 
    // from the remaining_accounts in the Context
    let mut libraries = Vec::new();
    
    for account_info in ctx.remaining_accounts {
        // Skip if account doesn't belong to this program
        if account_info.owner != &program_id {
            continue;
        }
        
        // Check if account data starts with LibraryInfo discriminator
        let data = account_info.try_borrow_data()?;
        if data.len() < 8 || data[0..8] != *discriminator {
            continue;
        }
        
        // Deserialize the account
        // The try_from_slice function starts after the discriminator
        let library_info = match LibraryInfo::try_deserialize(&mut &data[..]) {
            Ok(info) => info,
            Err(_) => continue,
        };
        
        // Apply start_after filter if provided
        if let Some(start_after_key) = start_after {
            if libraries.is_empty() && library_info.program_id <= start_after_key {
                continue;
            }
        }
        
        // Convert to response type
        let response = LibraryInfoResponse::from(&library_info);
        libraries.push(response);
        
        // Check if we've reached the limit
        if libraries.len() >= limit as usize {
            break;
        }
    }
    
    // Sort by program ID for consistent ordering
    libraries.sort_by(|a, b| a.program_id.to_bytes().cmp(&b.program_id.to_bytes()));
    
    msg!("Found {} libraries", libraries.len());
    Ok(libraries)
} 