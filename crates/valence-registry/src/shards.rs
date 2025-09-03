// Shard registry aligned with valence-kernel architecture
//
// This module provides client-side helpers for tracking shard deployments
// and their integration with the valence-kernel function registry.

use crate::error::{RegistryError, Result};
use anchor_lang::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ================================
// Shard Registry Structures
// ================================

/// Shard metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShardMetadata {
    /// Shard name
    pub name: String,
    /// Shard version
    pub version: String,
    /// Shard description
    pub description: String,
    /// Official website
    pub website: Option<String>,
    /// Source code repository
    pub repository: Option<String>,
    /// Security audits
    pub audits: Vec<AuditInfo>,
}

/// Security audit information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditInfo {
    /// Auditor name
    pub auditor: String,
    /// Audit report URL
    pub report_url: String,
    /// Audit date
    pub audit_date: i64,
    /// Audit score/rating
    pub score: Option<String>,
}

/// Shard instance deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardInstance {
    /// Unique shard identifier
    pub shard_id: [u8; 32],
    /// Deployed program ID on Solana
    pub program_id: Pubkey,
    /// Deployment authority
    pub authority: Pubkey,
    /// Shard metadata
    pub metadata: ShardMetadata,
    /// Interface definition (simplified)
    pub interface: ShardInterface,
    /// Deployment timestamp
    pub deployed_at: i64,
    /// Network identifier (mainnet, devnet, etc.)
    pub network: String,
    /// Functions this shard provides
    pub provided_functions: Vec<u64>, // Registry IDs
    /// Functions this shard requires
    pub required_functions: Vec<u64>, // Registry IDs
}

/// Simplified shard interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardInterface {
    /// Available instructions
    pub instructions: Vec<String>,
    /// Account types
    pub accounts: Vec<String>,
    /// Error codes
    pub errors: Vec<String>,
}

impl ShardInstance {
    /// Create a new shard instance
    pub fn new(
        program_id: Pubkey,
        authority: Pubkey,
        metadata: ShardMetadata,
        network: String,
    ) -> Self {
        let shard_id = Self::compute_id(&metadata.name, &metadata.version);
        
        Self {
            shard_id,
            program_id,
            authority,
            metadata,
            interface: ShardInterface {
                instructions: vec![],
                accounts: vec![],
                errors: vec![],
            },
            deployed_at: chrono::Utc::now().timestamp(),
            network,
            provided_functions: vec![],
            required_functions: vec![],
        }
    }
    
    /// Compute shard ID from name and version
    pub fn compute_id(name: &str, version: &str) -> [u8; 32] {
        blake3::hash(format!("{}:{}", name, version).as_bytes()).into()
    }
    
    /// Add a provided function
    pub fn add_provided_function(&mut self, registry_id: u64) {
        if !self.provided_functions.contains(&registry_id) {
            self.provided_functions.push(registry_id);
        }
    }
    
    /// Add a required function
    pub fn add_required_function(&mut self, registry_id: u64) {
        if !self.required_functions.contains(&registry_id) {
            self.required_functions.push(registry_id);
        }
    }
}

// ================================
// Shard Registry Implementation
// ================================

/// In-memory shard registry for client-side operations
pub struct ShardRegistry {
    /// Shards by program ID
    shards: HashMap<Pubkey, ShardInstance>,
    /// Index by shard ID
    shard_index: HashMap<[u8; 32], Pubkey>,
    /// Index by network
    network_index: HashMap<String, Vec<Pubkey>>,
    /// Index by provided functions
    function_providers: HashMap<u64, Vec<Pubkey>>,
}

impl Default for ShardRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ShardRegistry {
    /// Create a new shard registry
    pub fn new() -> Self {
        Self {
            shards: HashMap::new(),
            shard_index: HashMap::new(),
            network_index: HashMap::new(),
            function_providers: HashMap::new(),
        }
    }
    
    /// Register a shard
    pub fn register_shard(&mut self, instance: ShardInstance) -> Result<()> {
        let program_id = instance.program_id;
        
        // Check for duplicates
        if self.shards.contains_key(&program_id) {
            return Err(RegistryError::ShardAlreadyExists);
        }
        
        // Update indices
        self.shard_index.insert(instance.shard_id, program_id);
        
        self.network_index
            .entry(instance.network.clone())
            .or_default()
            .push(program_id);
            
        for &function_id in &instance.provided_functions {
            self.function_providers
                .entry(function_id)
                .or_default()
                .push(program_id);
        }
        
        // Store shard
        self.shards.insert(program_id, instance);
        
        Ok(())
    }
    
    /// Get shard by program ID
    pub fn get_shard(&self, program_id: &Pubkey) -> Option<&ShardInstance> {
        self.shards.get(program_id)
    }
    
    /// Get shard by shard ID
    pub fn get_shard_by_id(&self, shard_id: &[u8; 32]) -> Option<&ShardInstance> {
        if let Some(program_id) = self.shard_index.get(shard_id) {
            self.shards.get(program_id)
        } else {
            None
        }
    }
    
    /// List shards by network
    pub fn list_by_network(&self, network: &str) -> Vec<&ShardInstance> {
        if let Some(program_ids) = self.network_index.get(network) {
            program_ids
                .iter()
                .filter_map(|id| self.shards.get(id))
                .collect()
        } else {
            vec![]
        }
    }
    
    /// Find shards that provide a specific function
    pub fn find_function_providers(&self, function_id: u64) -> Vec<&ShardInstance> {
        if let Some(program_ids) = self.function_providers.get(&function_id) {
            program_ids
                .iter()
                .filter_map(|id| self.shards.get(id))
                .collect()
        } else {
            vec![]
        }
    }
    
    /// Get shard count
    pub fn shard_count(&self) -> usize {
        self.shards.len()
    }
    
    /// List all shards
    pub fn list_all_shards(&self) -> Vec<&ShardInstance> {
        self.shards.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_shard_metadata() -> ShardMetadata {
        ShardMetadata {
            name: "test_shard".to_string(),
            version: "1.0.0".to_string(),
            description: "A test shard".to_string(),
            website: Some("https://test.com".to_string()),
            repository: Some("https://github.com/test/test".to_string()),
            audits: vec![AuditInfo {
                auditor: "Test Auditor".to_string(),
                report_url: "https://audit.test".to_string(),
                audit_date: chrono::Utc::now().timestamp(),
                score: Some("A".to_string()),
            }],
        }
    }

    #[test]
    fn test_shard_registry() {
        let mut registry = ShardRegistry::new();
        
        let program_id = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        
        let mut instance = ShardInstance::new(
            program_id,
            authority,
            mock_shard_metadata(),
            "localnet".to_string(),
        );
        
        // Add some functions
        instance.add_provided_function(1001);
        instance.add_required_function(1002);
        
        // Register shard
        let result = registry.register_shard(instance.clone());
        assert!(result.is_ok());
        
        // Retrieve shard
        let retrieved = registry.get_shard(&program_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().metadata.name, "test_shard");
        
        // Test function provider lookup
        let providers = registry.find_function_providers(1001);
        assert_eq!(providers.len(), 1);
        
        // Test network lookup
        let localnet_shards = registry.list_by_network("localnet");
        assert_eq!(localnet_shards.len(), 1);
        
        // Test shard count
        assert_eq!(registry.shard_count(), 1);
    }

    #[test]
    fn test_shard_id_computation() {
        let id1 = ShardInstance::compute_id("test", "1.0.0");
        let id2 = ShardInstance::compute_id("test", "1.0.0");
        let id3 = ShardInstance::compute_id("test", "1.0.1");
        
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }
}