use anchor_lang::prelude::*;
use std::collections::HashMap;
use crate::state::Authorization;

/// Cache structure for authorization lookups
/// This is a zero-copy implementation to avoid cloning
#[derive(Default)]
pub struct AuthorizationCache {
    /// Maps authorization labels to their account addresses
    pub label_to_address: HashMap<String, Pubkey>,
    /// Cached bump seeds for derived PDAs
    pub pda_bumps: HashMap<String, u8>,
}

impl AuthorizationCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            label_to_address: HashMap::new(),
            pda_bumps: HashMap::new(),
        }
    }
    
    /// Add an authorization to the cache
    pub fn add_authorization(&mut self, authorization: &Authorization) {
        let label = authorization.label.clone();
        // Derive the PDA address for the authorization
        let (address, _) = Pubkey::find_program_address(
            &[b"authorization", label.as_bytes()],
            &crate::ID,
        );
        
        self.label_to_address.insert(label.clone(), address);
        self.pda_bumps.insert(label, authorization.bump);
    }
    
    /// Get the address of an authorization by label with O(1) complexity
    pub fn get_address(&self, label: &str) -> Option<Pubkey> {
        self.label_to_address.get(label).copied()
    }
    
    /// Get the bump seed for an authorization PDA
    pub fn get_bump(&self, label: &str) -> Option<u8> {
        self.pda_bumps.get(label).copied()
    }
    
    /// Remove an authorization from the cache
    pub fn remove_authorization(&mut self, label: &str) {
        self.label_to_address.remove(label);
        self.pda_bumps.remove(label);
    }
    
    /// Check if an authorization is in the cache
    pub fn contains(&self, label: &str) -> bool {
        self.label_to_address.contains_key(label)
    }
    
    /// Get the number of cached authorizations
    pub fn len(&self) -> usize {
        self.label_to_address.len()
    }
    
    /// Clear the cache
    pub fn clear(&mut self) {
        self.label_to_address.clear();
        self.pda_bumps.clear();
    }
}

/// Helper functions for efficient authorization lookup
pub mod helpers {
    use super::*;
    
    /// Find an authorization account efficiently
    /// First checks the cache, then falls back to on-chain lookup
    pub fn find_authorization(
        cache: &mut AuthorizationCache, 
        label: &str,
        program_id: &Pubkey,
    ) -> Result<(Pubkey, u8)> {
        // Check cache first for O(1) lookup
        if let Some(address) = cache.get_address(label) {
            if let Some(bump) = cache.get_bump(label) {
                return Ok((address, bump));
            }
        }
        
        // Fall back to computing the PDA
        let (address, bump) = Pubkey::find_program_address(
            &[b"authorization", label.as_bytes()],
            program_id,
        );
        
        // Add to cache for future lookups
        cache.label_to_address.insert(label.to_string(), address);
        cache.pda_bumps.insert(label.to_string(), bump);
        
        Ok((address, bump))
    }
    
    /// Prefetch a batch of authorizations in one operation
    /// This is useful when processing multiple authorizations
    pub fn prefetch_authorizations<'info>(
        cache: &mut AuthorizationCache,
        labels: &[&str],
        accounts: &[AccountInfo<'info>],
    ) -> Result<()> {
        for label in labels {
            for account in accounts {
                // Try to find the account for this label
                let (expected_address, _) = Pubkey::find_program_address(
                    &[b"authorization", label.as_bytes()],
                    &crate::ID,
                );
                
                if account.key() == expected_address {
                    if let Ok(authorization) = Account::<Authorization>::try_from(account) {
                        cache.add_authorization(&authorization);
                        break;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get the current cache statistics
    pub fn get_cache_stats(cache: &AuthorizationCache) -> CacheStats {
        CacheStats {
            cached_authorizations: cache.len(),
            memory_usage: estimate_memory_usage(cache),
        }
    }
    
    /// Estimate memory usage of the cache in bytes
    fn estimate_memory_usage(cache: &AuthorizationCache) -> usize {
        let mut size = 0;
        
        // Size of the hashmaps
        size += std::mem::size_of::<HashMap<String, Pubkey>>();
        size += std::mem::size_of::<HashMap<String, u8>>();
        
        // Size of the keys and values
        for (key, _) in &cache.label_to_address {
            size += key.len() + std::mem::size_of::<String>();
            size += std::mem::size_of::<Pubkey>();
        }
        
        for (key, _) in &cache.pda_bumps {
            size += key.len() + std::mem::size_of::<String>();
            size += std::mem::size_of::<u8>();
        }
        
        size
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    /// Number of cached authorizations
    pub cached_authorizations: usize,
    /// Estimated memory usage in bytes
    pub memory_usage: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_basic_operations() {
        let mut cache = AuthorizationCache::new();
        
        // Create a mock authorization
        let mut auth = Authorization {
            label: "test_auth".to_string(),
            owner: Pubkey::new_unique(),
            is_active: true,
            permission_type: crate::state::PermissionType::Public,
            allowed_users: vec![],
            not_before: 0,
            expiration: None,
            max_concurrent_executions: 10,
            priority: crate::state::Priority::Medium,
            subroutine_type: crate::state::SubroutineType::Atomic,
            current_executions: 0,
            bump: 254,
        };
        
        // Add to cache
        cache.add_authorization(&auth);
        
        // Check it was added
        assert!(cache.contains("test_auth"));
        assert_eq!(cache.len(), 1);
        
        // Get the address
        let addr = cache.get_address("test_auth");
        assert!(addr.is_some());
        
        // Get the bump
        let bump = cache.get_bump("test_auth");
        assert_eq!(bump, Some(254));
        
        // Add another authorization
        auth.label = "test_auth2".to_string();
        cache.add_authorization(&auth);
        
        // Check both are in the cache
        assert_eq!(cache.len(), 2);
        assert!(cache.contains("test_auth"));
        assert!(cache.contains("test_auth2"));
        
        // Remove one
        cache.remove_authorization("test_auth");
        assert_eq!(cache.len(), 1);
        assert!(!cache.contains("test_auth"));
        assert!(cache.contains("test_auth2"));
        
        // Clear the cache
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(!cache.contains("test_auth2"));
    }
    
    #[test]
    fn test_cache_stats() {
        let mut cache = AuthorizationCache::new();
        
        // Empty cache stats
        let stats = helpers::get_cache_stats(&cache);
        assert_eq!(stats.cached_authorizations, 0);
        
        // Add an authorization
        let auth = Authorization {
            label: "test_auth".to_string(),
            owner: Pubkey::new_unique(),
            is_active: true,
            permission_type: crate::state::PermissionType::Public,
            allowed_users: vec![],
            not_before: 0,
            expiration: None,
            max_concurrent_executions: 10,
            priority: crate::state::Priority::Medium,
            subroutine_type: crate::state::SubroutineType::Atomic,
            current_executions: 0,
            bump: 254,
        };
        
        cache.add_authorization(&auth);
        
        // Check stats after adding
        let stats = helpers::get_cache_stats(&cache);
        assert_eq!(stats.cached_authorizations, 1);
        assert!(stats.memory_usage > 0);
    }
} 