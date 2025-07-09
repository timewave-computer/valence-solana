/// Type definitions for Valence Protocol SDK
/// 
/// This module contains all the types used throughout the SDK for interacting
/// with the Valence Protocol programs.

use anchor_lang::prelude::*;
use serde::{Deserialize, Serialize};
use solana_sdk::{commitment_config::CommitmentConfig, signature::Keypair};

/// Program IDs for the Valence Protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramIds {
    pub kernel: Pubkey,         // Main kernel program (includes shards with embedded eval)
    pub processor: Pubkey,      // Processor singleton
    pub scheduler: Pubkey,      // Scheduler singleton
    pub diff: Pubkey, // Diff singleton
    pub registry: Pubkey,       // Registry (if still needed)
}

impl Default for ProgramIds {
    fn default() -> Self {
        Self {
            kernel: Pubkey::new_unique(),         // Replace with actual program IDs
            processor: Pubkey::new_unique(),
            scheduler: Pubkey::new_unique(),
            diff: Pubkey::new_unique(),
            registry: Pubkey::new_unique(),
        }
    }
}

/// SDK Configuration
#[derive(Debug)]
pub struct ValenceConfig {
    pub program_ids: ProgramIds,
    pub cluster: anchor_client::Cluster,
    pub payer: Keypair,
    pub commitment: Option<CommitmentConfig>,
}

impl ValenceConfig {
    /// Create a new config for localhost/test
    pub fn localhost() -> Self {
        Self {
            program_ids: ProgramIds::default(),
            cluster: anchor_client::Cluster::Localnet,
            payer: Keypair::new(),
            commitment: Some(CommitmentConfig::confirmed()),
        }
    }
    
    /// Create a new config for devnet
    pub fn devnet() -> Self {
        Self {
            program_ids: ProgramIds::default(),
            cluster: anchor_client::Cluster::Devnet,
            payer: Keypair::new(),
            commitment: Some(CommitmentConfig::confirmed()),
        }
    }
}

/// Execution context for capability execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValenceExecutionContext {
    pub capability_id: String,
    pub session: Pubkey,
    pub caller: Pubkey,
    pub input_data: Vec<u8>,
    pub block_height: u64,
    pub timestamp: i64,
    pub compute_limit: Option<u32>,
    pub parameters: std::collections::BTreeMap<String, serde_json::Value>,
    pub labels: Vec<String>,
}

impl ValenceExecutionContext {
    pub fn new(capability_id: String, session: Pubkey, caller: Pubkey) -> Self {
        Self {
            capability_id,
            session,
            caller,
            input_data: Vec::new(),
            block_height: 0,
            timestamp: 0,
            compute_limit: None,
            parameters: std::collections::BTreeMap::new(),
            labels: Vec::new(),
        }
    }
    
    pub fn with_input_data(mut self, data: Vec<u8>) -> Self {
        self.input_data = data;
        self
    }
    
    pub fn with_compute_limit(mut self, limit: u32) -> Self {
        self.compute_limit = Some(limit);
        self
    }
    
    pub fn with_labels(mut self, labels: Vec<String>) -> Self {
        self.labels = labels;
        self
    }
}

/// Session metadata for session management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub description: String,
    pub tags: Vec<String>,
    pub max_lifetime: i64, // 0 = unlimited
}

impl Default for SessionMetadata {
    fn default() -> Self {
        Self {
            description: String::new(),
            tags: Vec::new(),
            max_lifetime: 0,
        }
    }
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub owner: Pubkey,
    pub is_active: bool,
    pub capabilities: Vec<String>,
    pub namespaces: Vec<String>,
    pub metadata: SessionMetadata,
    pub created_at: i64,
    pub last_updated: i64,
    pub version: u64,
}

/// Capability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub capability_id: String,
    pub shard: Pubkey,
    pub verification_functions: Vec<[u8; 32]>,
    pub description: String,
    pub is_active: bool,
    pub total_executions: u64,
    pub last_execution_block_height: u64,
    pub last_execution_timestamp: i64,
}

/// Library registry types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LibraryStatus {
    Draft,
    Published,
    Deprecated,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZkProgramStatus {
    Draft,
    Published,
    Verified,
    Deprecated,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Runtime,
    Development,
    Optional,
    Peer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub library_id: String,
    pub name: String,
    pub version: String,
    pub author: Pubkey,
    pub metadata_hash: [u8; 32],
    pub program_id: Pubkey,
    pub status: LibraryStatus,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
    pub is_verified: bool,
    pub usage_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProgramEntry {
    pub program_id: String,
    pub name: String,
    pub version: String,
    pub author: Pubkey,
    pub verification_key: Vec<u8>,
    pub metadata_hash: [u8; 32],
    pub status: ZkProgramStatus,
    pub is_verified: bool,
    pub constraints_count: u64,
    pub proofs_verified: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEntry {
    pub dependent_library: String,
    pub dependency_library: String,
    pub version_requirement: String,
    pub is_optional: bool,
    pub dependency_type: DependencyType,
}

/// Transaction and execution result types
#[derive(Debug, Clone)]
pub struct TransactionResult {
    pub signature: solana_sdk::signature::Signature,
    pub success: bool,
    pub error: Option<String>,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub transaction_result: TransactionResult,
    pub execution_id: Option<u64>,
    pub capability_id: String,
    pub session: Pubkey,
}

/// Execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub max_execution_time: Option<u64>, // seconds
    pub max_compute_units: Option<u32>,
    pub record_execution: bool,
    pub parameters: Option<Vec<u8>>,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_execution_time: None,
            max_compute_units: None,
            record_execution: true,
            parameters: None,
        }
    }
}

/// Utility types for pagination and queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationOptions {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>, // "asc" or "desc"
}

impl Default for PaginationOptions {
    fn default() -> Self {
        Self {
            page: Some(1),
            page_size: Some(50),
            sort_by: None,
            sort_order: Some("asc".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult<T> {
    pub items: Vec<T>,
    pub total_count: u64,
    pub page: u64,
    pub page_size: u64,
    pub has_more: bool,
}

/// Event types for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityExecutedEvent {
    pub capability_id: String,
    pub session: Pubkey,
    pub executor: Pubkey,
    pub success: bool,
    pub block_height: u64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCreatedEvent {
    pub session_id: String,
    pub owner: Pubkey,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryRegisteredEvent {
    pub library_id: String,
    pub name: String,
    pub version: String,
    pub author: Pubkey,
    pub timestamp: i64,
}

/// Builder pattern interfaces for complex types
pub struct CapabilityBuilder {
    capability: Capability,
}

impl CapabilityBuilder {
    pub fn new(capability_id: String, shard: Pubkey) -> Self {
        Self {
            capability: Capability {
                capability_id,
                shard,
                verification_functions: Vec::new(),
                description: String::new(),
                is_active: true,
                total_executions: 0,
                last_execution_block_height: 0,
                last_execution_timestamp: 0,
            },
        }
    }
    
    pub fn with_description(mut self, description: String) -> Self {
        self.capability.description = description;
        self
    }
    
    pub fn with_verification_functions(mut self, functions: Vec<[u8; 32]>) -> Self {
        self.capability.verification_functions = functions;
        self
    }
    
    pub fn build(self) -> Capability {
        self.capability
    }
}

pub struct SessionBuilder {
    session: Session,
}

impl SessionBuilder {
    pub fn new(session_id: String, owner: Pubkey) -> Self {
        Self {
            session: Session {
                session_id,
                owner,
                is_active: true,
                capabilities: Vec::new(),
                namespaces: Vec::new(),
                metadata: SessionMetadata::default(),
                created_at: chrono::Utc::now().timestamp(),
                last_updated: chrono::Utc::now().timestamp(),
                version: 1,
            },
        }
    }
    
    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.session.capabilities = capabilities;
        self
    }
    
    pub fn with_namespaces(mut self, namespaces: Vec<String>) -> Self {
        self.session.namespaces = namespaces;
        self
    }
    
    pub fn with_metadata(mut self, metadata: SessionMetadata) -> Self {
        self.session.metadata = metadata;
        self
    }
    
    pub fn build(self) -> Session {
        self.session
    }
}

/// Session information type
pub type SessionInfo = Session;

/// Execution request for batch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub capability_id: String,
    pub input_data: Vec<u8>,
    pub target_session: Option<Pubkey>,
    pub config: Option<ExecutionConfig>,
}

/// Library metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryMetadata {
    pub description: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
    pub tags: Vec<String>,
}

/// ZK program metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProgramMetadata {
    pub description: String,
    pub proving_system: String,
    pub verification_key_size: usize,
    pub circuit_size: Option<u64>,
    pub tags: Vec<String>,
}

/// Library filter options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryFilter {
    pub status: Option<LibraryStatus>,
    pub author: Option<Pubkey>,
    pub tags: Option<Vec<String>>,
    pub verified_only: bool,
}

/// ZK program filter options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProgramFilter {
    pub status: Option<ZkProgramStatus>,
    pub author: Option<Pubkey>,
    pub proving_system: Option<String>,
    pub verified_only: bool,
}

/// Library information
pub type LibraryInfo = LibraryEntry;

/// ZK program information
pub type ZkProgramInfo = ZkProgramEntry;

/// Runtime statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStats {
    pub total_capabilities: u64,
    pub total_sessions: u64,
    pub total_executions: u64,
    pub total_libraries: u64,
    pub active_sessions: u64,
    pub processor_status: ProcessorStats,
    pub scheduler_status: SchedulerStats,
    pub diff_status: DiffStats,
}

/// Processor statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorStats {
    pub is_paused: bool,
    pub total_processed: u64,
    pub total_failed: u64,
    pub average_processing_time_ms: u64,
}

/// Scheduler statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStats {
    pub queue_size: u32,
    pub total_scheduled: u64,
    pub total_executed: u64,
    pub average_wait_time_ms: u64,
}

/// Diff statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStats {
    pub total_diffs_processed: u64,
    pub total_diffs_verified: u64,
    pub average_diff_size_bytes: u64,
}

/// Network status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub connected: bool,
    pub cluster: String,
    pub block_height: u64,
    pub block_time: i64,
    pub tps: f64,
    pub node_version: String,
} 