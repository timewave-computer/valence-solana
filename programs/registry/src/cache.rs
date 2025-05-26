use anchor_lang::prelude::*;
use std::collections::HashMap;
use crate::state::LibraryInfo;

/// Cache structure for library lookups
/// This is a zero-copy implementation to avoid cloning
#[derive(Default)]
pub struct LibraryCache {
    /// Maps program IDs to their library info addresses
    pub program_id_to_address: HashMap<Pubkey, Pubkey>,
    /// Cached bump seeds for derived PDAs
    pub pda_bumps: HashMap<Pubkey, u8>,
    /// Approval status cache to avoid account lookups
    pub approval_status: HashMap<Pubkey, bool>,
}

impl LibraryCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            program_id_to_address: HashMap::new(),
            pda_bumps: HashMap::new(),
            approval_status: HashMap::new(),
        }
    }
    
    /// Add a library to the cache
    pub fn add_library(&mut self, library_info: &LibraryInfo) {
        let program_id = library_info.program_id;
        
        // Derive the PDA address for the library
        let (address, _) = Pubkey::find_program_address(
            &[b"library_info", program_id.as_ref()],
            &crate::ID,
        );
        
        self.program_id_to_address.insert(program_id, address);
        self.pda_bumps.insert(program_id, library_info.bump);
        self.approval_status.insert(program_id, library_info.is_approved);
    }
    
    /// Get the address of a library by program ID with O(1) complexity
    pub fn get_address(&self, program_id: &Pubkey) -> Option<Pubkey> {
        self.program_id_to_address.get(program_id).copied()
    }
    
    /// Get the bump seed for a library PDA
    pub fn get_bump(&self, program_id: &Pubkey) -> Option<u8> {
        self.pda_bumps.get(program_id).copied()
    }
    
    /// Get the approval status of a library (cached)
    pub fn is_approved(&self, program_id: &Pubkey) -> Option<bool> {
        self.approval_status.get(program_id).copied()
    }
    
    /// Remove a library from the cache
    pub fn remove_library(&mut self, program_id: &Pubkey) {
        self.program_id_to_address.remove(program_id);
        self.pda_bumps.remove(program_id);
        self.approval_status.remove(program_id);
    }
    
    /// Update the approval status of a library in the cache
    pub fn update_approval_status(&mut self, program_id: &Pubkey, is_approved: bool) {
        if self.program_id_to_address.contains_key(program_id) {
            self.approval_status.insert(*program_id, is_approved);
        }
    }
    
    /// Check if a library is in the cache
    pub fn contains(&self, program_id: &Pubkey) -> bool {
        self.program_id_to_address.contains_key(program_id)
    }
    
    /// Get the number of cached libraries
    pub fn len(&self) -> usize {
        self.program_id_to_address.len()
    }
    
    /// Clear the cache
    pub fn clear(&mut self) {
        self.program_id_to_address.clear();
        self.pda_bumps.clear();
        self.approval_status.clear();
    }
}

/// Helper functions for efficient library lookup
pub mod helpers {
    use super::*;
    
    /// Find a library account efficiently
    /// First checks the cache, then falls back to on-chain lookup
    pub fn find_library(
        cache: &mut LibraryCache, 
        program_id: &Pubkey,
    ) -> Result<(Pubkey, u8)> {
        // Check cache first for O(1) lookup
        if let Some(address) = cache.get_address(program_id) {
            if let Some(bump) = cache.get_bump(program_id) {
                return Ok((address, bump));
            }
        }
        
        // Fall back to computing the PDA
        let (address, bump) = Pubkey::find_program_address(
            &[b"library_info", program_id.as_ref()],
            &crate::ID,
        );
        
        // Add to cache for future lookups
        cache.program_id_to_address.insert(*program_id, address);
        cache.pda_bumps.insert(*program_id, bump);
        
        Ok((address, bump))
    }
    
    /// Check if a library is approved (cached)
    pub fn is_library_approved(
        cache: &LibraryCache,
        program_id: &Pubkey,
    ) -> Option<bool> {
        cache.is_approved(program_id)
    }
    
    /// Prefetch a batch of libraries in one operation
    pub fn prefetch_libraries<'info>(
        cache: &mut LibraryCache,
        program_ids: &[Pubkey],
        accounts: &'info [AccountInfo<'info>],
    ) -> Result<()> {
        for program_id in program_ids {
            for account in accounts {
                // Try to find the account for this program ID
                let (expected_address, _) = Pubkey::find_program_address(
                    &[b"library_info", program_id.as_ref()],
                    &crate::ID,
                );
                
                if account.key() == expected_address {
                    if let Ok(library_info) = Account::<LibraryInfo>::try_from(account) {
                        cache.add_library(&library_info);
                        break;
                    }
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_basic_operations() {
        let mut cache = LibraryCache::new();
        
        // Create a mock library
        let mut library = LibraryInfo {
            program_id: Pubkey::new_unique(),
            library_type: "token_transfer".to_string(),
            description: "A test library".to_string(),
            is_approved: true,
            version: "1.0.0".to_string(),
            last_updated: 0,
            dependencies: Vec::new(),
            bump: 254,
        };
        
        // Add to cache
        cache.add_library(&library);
        
        // Check it was added
        assert!(cache.contains(&library.program_id));
        assert_eq!(cache.len(), 1);
        
        // Get the address
        let addr = cache.get_address(&library.program_id);
        assert!(addr.is_some());
        
        // Get the approval status
        let is_approved = cache.is_approved(&library.program_id);
        assert_eq!(is_approved, Some(true));
        
        // Update approval status
        cache.update_approval_status(&library.program_id, false);
        assert_eq!(cache.is_approved(&library.program_id), Some(false));
        
        // Add another library
        library.program_id = Pubkey::new_unique();
        cache.add_library(&library);
        
        // Check both are in the cache
        assert_eq!(cache.len(), 2);
        
        // Remove one
        cache.remove_library(&library.program_id);
        assert_eq!(cache.len(), 1);
        
        // Clear the cache
        cache.clear();
        assert_eq!(cache.len(), 0);
    }
} 