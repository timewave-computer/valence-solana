use crate::error::{RegistryError, Result};
use anchor_lang::prelude::*;
use serde::{Deserialize, Serialize};

// ================================
// Protocol Registry
// ================================

/// Protocol instance registry entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolInstance {
    /// Protocol ID (function content hash)
    pub protocol_id: [u8; 32],

    /// Deployed program ID
    pub program_id: Pubkey,

    /// Deployment authority
    pub authority: Pubkey,

    /// Instance metadata
    pub metadata: ProtocolMetadata,

    /// IDL (Interface Definition Language)
    pub idl: serde_json::Value,

    /// Deployment timestamp
    pub deployed_at: i64,

    /// Network (mainnet, devnet, etc)
    pub network: String,
}

/// Protocol metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMetadata {
    /// Protocol name
    pub name: String,

    /// Protocol version
    pub version: String,

    /// Brief description
    pub description: String,

    /// Protocol website
    pub website: Option<String>,

    /// Source code repository
    pub repository: Option<String>,

    /// Audit reports
    pub audits: Vec<AuditReport>,
}

/// Audit report reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    /// Auditor name
    pub auditor: String,

    /// Report URL
    pub report_url: String,

    /// Audit date
    pub audit_date: i64,
}

// ================================
// Protocol Registry Service
// ================================

#[async_trait::async_trait]
pub trait ProtocolRegistry: Send + Sync {
    /// Register a protocol instance
    async fn register_protocol(&self, instance: ProtocolInstance) -> Result<()>;

    /// Get protocol by program ID
    async fn get_protocol(&self, program_id: &Pubkey) -> Result<ProtocolInstance>;

    /// Search protocols by function ID
    async fn search_by_function(&self, function_id: &[u8; 32]) -> Result<Vec<ProtocolInstance>>;

    /// List protocols by network
    async fn list_by_network(&self, network: &str) -> Result<Vec<ProtocolInstance>>;

    /// Get protocol IDL
    async fn get_idl(&self, program_id: &Pubkey) -> Result<serde_json::Value>;
}

// ================================
// In-Memory Protocol Registry
// ================================

pub struct InMemoryProtocolRegistry {
    protocols: std::sync::RwLock<std::collections::HashMap<Pubkey, ProtocolInstance>>,
}

impl Default for InMemoryProtocolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryProtocolRegistry {
    pub fn new() -> Self {
        Self {
            protocols: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl ProtocolRegistry for InMemoryProtocolRegistry {
    async fn register_protocol(&self, instance: ProtocolInstance) -> Result<()> {
        let mut protocols = self.protocols.write().unwrap();
        protocols.insert(instance.program_id, instance);
        Ok(())
    }

    async fn get_protocol(&self, program_id: &Pubkey) -> Result<ProtocolInstance> {
        let protocols = self.protocols.read().unwrap();
        protocols
            .get(program_id)
            .cloned()
            .ok_or_else(|| RegistryError::ProtocolNotFound(program_id.to_string()))
    }

    async fn search_by_function(&self, function_id: &[u8; 32]) -> Result<Vec<ProtocolInstance>> {
        let protocols = self.protocols.read().unwrap();
        let results: Vec<_> = protocols
            .values()
            .filter(|instance| instance.protocol_id == *function_id)
            .cloned()
            .collect();
        Ok(results)
    }

    async fn list_by_network(&self, network: &str) -> Result<Vec<ProtocolInstance>> {
        let protocols = self.protocols.read().unwrap();
        let results: Vec<_> = protocols
            .values()
            .filter(|instance| instance.network == network)
            .cloned()
            .collect();
        Ok(results)
    }

    async fn get_idl(&self, program_id: &Pubkey) -> Result<serde_json::Value> {
        let protocol = self.get_protocol(program_id).await?;
        Ok(protocol.idl)
    }
}
