// Function registry aligned with valence-kernel and valence-functions design
//
// This module provides client-side helpers for working with the kernel's
// hardcoded function registry and valence-functions implementations.

use crate::error::{RegistryError, Result};
use anchor_lang::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use lru::LruCache;
use std::num::NonZeroUsize;

// ================================
// Function Registry Structures
// ================================

/// Function information matching the kernel's hardcoded registry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionInfo {
    /// Registry ID used by the kernel
    pub registry_id: u64,
    /// Program ID that implements this function
    pub program_id: Pubkey,
    /// Function name
    pub name: String,
    /// Function version
    pub version: u16,
    /// Whether this function is currently active
    pub is_active: bool,
    /// Estimated compute units for this function
    pub compute_units: u64,
}

impl FunctionInfo {
    /// Create a new function info entry
    pub fn new(
        registry_id: u64,
        program_id: Pubkey,
        name: String,
        version: u16,
        compute_units: u64,
    ) -> Self {
        Self {
            registry_id,
            program_id,
            name,
            version,
            is_active: true,
            compute_units,
        }
    }
    
    /// Get the function signature for verification
    pub fn signature(&self) -> String {
        format!("{}:{}:{}", self.name, self.version, self.program_id)
    }
}

/// Function entry from valence-functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionEntry {
    /// Function metadata
    pub info: FunctionInfo,
    /// Content hash for integrity verification
    pub content_hash: [u8; 32],
    /// Function description
    pub description: String,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Registration timestamp
    pub registered_at: i64,
}

impl FunctionEntry {
    /// Create a new function entry
    pub fn new(info: FunctionInfo, description: String, tags: Vec<String>) -> Self {
        let content_hash = Self::compute_hash(&format!("{}:{}", info.name, info.version));
        
        Self {
            info,
            content_hash,
            description,
            tags,
            registered_at: chrono::Utc::now().timestamp(),
        }
    }
    
    /// Compute content hash for a function
    pub fn compute_hash(content: &str) -> [u8; 32] {
        blake3::hash(content.as_bytes()).into()
    }
    
    /// Verify the content hash
    pub fn verify_hash(&self) -> bool {
        let expected = Self::compute_hash(&format!("{}:{}", self.info.name, self.info.version));
        self.content_hash == expected
    }
}

// ================================
// Function Registry Implementation
// ================================

/// In-memory function registry for client-side operations
pub struct FunctionRegistry {
    /// Functions by registry ID
    functions: HashMap<u64, FunctionEntry>,
    /// LRU cache for quick lookups
    cache: LruCache<u64, FunctionEntry>,
    /// Index by program ID
    program_index: HashMap<Pubkey, Vec<u64>>,
    /// Index by tags
    tag_index: HashMap<String, Vec<u64>>,
}

impl FunctionRegistry {
    /// Create a new function registry
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            cache: LruCache::new(NonZeroUsize::new(256).unwrap()),
            program_index: HashMap::new(),
            tag_index: HashMap::new(),
        }
    }
    
    /// Register a function
    pub fn register_function(&mut self, entry: FunctionEntry) -> Result<u64> {
        let registry_id = entry.info.registry_id;
        
        // Check for duplicates
        if self.functions.contains_key(&registry_id) {
            return Err(RegistryError::FunctionAlreadyExists);
        }
        
        // Verify hash
        if !entry.verify_hash() {
            return Err(RegistryError::InvalidContentHash);
        }
        
        // Update indices
        self.program_index
            .entry(entry.info.program_id)
            .or_default()
            .push(registry_id);
            
        for tag in &entry.tags {
            self.tag_index
                .entry(tag.clone())
                .or_default()
                .push(registry_id);
        }
        
        // Store function
        self.functions.insert(registry_id, entry.clone());
        self.cache.put(registry_id, entry);
        
        Ok(registry_id)
    }
    
    /// Get function by registry ID
    pub fn get_function(&mut self, registry_id: &u64) -> Option<FunctionEntry> {
        // Try cache first
        if let Some(entry) = self.cache.get(registry_id) {
            return Some(entry.clone());
        }
        
        // Fallback to main storage
        if let Some(entry) = self.functions.get(registry_id) {
            let cloned_entry = entry.clone();
            self.cache.put(*registry_id, cloned_entry.clone());
            Some(cloned_entry)
        } else {
            None
        }
    }
    
    /// List functions by program ID
    pub fn get_functions_by_program(&self, program_id: &Pubkey) -> Vec<&FunctionEntry> {
        if let Some(registry_ids) = self.program_index.get(program_id) {
            registry_ids
                .iter()
                .filter_map(|id| self.functions.get(id))
                .collect()
        } else {
            vec![]
        }
    }
    
    /// Search functions by tags
    pub fn search_by_tags(&self, tags: &[String]) -> Vec<&FunctionEntry> {
        let mut result_ids = std::collections::HashSet::new();
        
        for tag in tags {
            if let Some(ids) = self.tag_index.get(tag) {
                if result_ids.is_empty() {
                    result_ids.extend(ids);
                } else {
                    result_ids.retain(|id| ids.contains(id));
                }
            } else {
                return vec![]; // If any tag has no matches, return empty
            }
        }
        
        result_ids
            .iter()
            .filter_map(|id| self.functions.get(id))
            .collect()
    }
    
    /// List all active functions
    pub fn list_active_functions(&self) -> Vec<&FunctionEntry> {
        self.functions
            .values()
            .filter(|entry| entry.info.is_active)
            .collect()
    }
    
    /// Get function count
    pub fn function_count(&self) -> usize {
        self.functions.len()
    }
    
    /// Deactivate a function
    pub fn deactivate_function(&mut self, registry_id: &u64) -> Result<()> {
        if let Some(entry) = self.functions.get_mut(registry_id) {
            entry.info.is_active = false;
            // Update cache
            self.cache.put(*registry_id, entry.clone());
            Ok(())
        } else {
            Err(RegistryError::FunctionNotFound(registry_id.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_function_info(id: u64, name: &str) -> FunctionInfo {
        FunctionInfo::new(
            id,
            Pubkey::new_unique(),
            name.to_string(),
            1,
            10_000,
        )
    }

    #[test]
    fn test_function_registry() {
        let mut registry = FunctionRegistry::new();
        
        // Create test function
        let info = mock_function_info(1001, "test_function");
        let entry = FunctionEntry::new(
            info,
            "Test function".to_string(),
            vec!["test".to_string(), "example".to_string()],
        );
        
        // Register function
        let result = registry.register_function(entry.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1001);
        
        // Retrieve function
        let retrieved = registry.get_function(&1001);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().info.name, "test_function");
        
        // Test duplicate registration
        let duplicate = registry.register_function(entry);
        assert!(duplicate.is_err());
        
        // Test search by tags
        let results = registry.search_by_tags(&["test".to_string()]);
        assert_eq!(results.len(), 1);
        
        // Test function count
        assert_eq!(registry.function_count(), 1);
    }

    #[test]
    fn test_function_hash_verification() {
        let info = mock_function_info(1002, "hash_test");
        let entry = FunctionEntry::new(info, "Test".to_string(), vec![]);
        
        assert!(entry.verify_hash());
        
        // Test with modified entry
        let mut modified = entry.clone();
        modified.content_hash = [0u8; 32];
        assert!(!modified.verify_hash());
    }
}