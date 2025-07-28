//! Comprehensive audit trail system
//!
//! This module provides detailed audit logging capabilities for all security-sensitive
//! operations, maintaining an immutable trail of all activities.

use crate::{Result, RuntimeError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::{Mutex, RwLock};
use tracing::{error, info, warn};

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique entry ID
    pub id: String,

    /// Entry timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Event type
    pub event_type: AuditEventType,

    /// Actor (user, service, or system)
    pub actor: Actor,

    /// Resource being accessed
    pub resource: Option<Resource>,

    /// Operation details
    pub operation: Operation,

    /// Result of the operation
    pub result: OperationResult,

    /// Additional context
    pub context: HashMap<String, serde_json::Value>,

    /// Parent entry ID for linked events
    pub parent_id: Option<String>,

    /// Cryptographic hash of previous entry
    pub previous_hash: Option<String>,

    /// Hash of this entry
    pub entry_hash: String,
}

/// Type of audit event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType {
    /// Transaction construction
    TransactionConstruction,
    /// Transaction signing request
    SigningRequest,
    /// Transaction signed
    TransactionSigned,
    /// Transaction submission
    TransactionSubmission,
    /// Policy evaluation
    PolicyEvaluation,
    /// Security alert
    SecurityAlert,
    /// Configuration change
    ConfigurationChange,
    /// Access control
    AccessControl,
    /// System event
    SystemEvent,
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditEventType::TransactionConstruction => write!(f, "TRANSACTION_CONSTRUCTION"),
            AuditEventType::SigningRequest => write!(f, "SIGNING_REQUEST"),
            AuditEventType::TransactionSigned => write!(f, "TRANSACTION_SIGNED"),
            AuditEventType::TransactionSubmission => write!(f, "TRANSACTION_SUBMISSION"),
            AuditEventType::PolicyEvaluation => write!(f, "POLICY_EVALUATION"),
            AuditEventType::SecurityAlert => write!(f, "SECURITY_ALERT"),
            AuditEventType::ConfigurationChange => write!(f, "CONFIGURATION_CHANGE"),
            AuditEventType::AccessControl => write!(f, "ACCESS_CONTROL"),
            AuditEventType::SystemEvent => write!(f, "SYSTEM_EVENT"),
        }
    }
}

/// Actor performing the operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    /// Actor type
    pub actor_type: ActorType,

    /// Actor identifier
    pub id: String,

    /// Additional actor metadata
    pub metadata: HashMap<String, String>,
}

/// Type of actor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorType {
    User,
    Service,
    System,
    External,
}

/// Resource being accessed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Resource type
    pub resource_type: ResourceType,

    /// Resource identifier
    pub id: String,

    /// Additional attributes
    pub attributes: HashMap<String, String>,
}

/// Type of resource
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Account,
    Transaction,
    Program,
    Configuration,
    Key,
    Session,
}

/// Operation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Operation name
    pub name: String,

    /// Operation parameters
    pub parameters: HashMap<String, serde_json::Value>,

    /// Related transaction (if applicable)
    pub transaction_id: Option<String>,

    /// Related signature (if applicable)
    pub signature: Option<solana_sdk::signature::Signature>,
}

/// Operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationResult {
    Success {
        details: Option<serde_json::Value>,
    },
    Failure {
        error: String,
        error_code: Option<String>,
    },
    Partial {
        completed: Vec<String>,
        failed: Vec<String>,
    },
}

/// Audit logger implementation
pub struct AuditLogger {
    /// Storage backend
    storage: Arc<dyn AuditStorage>,

    /// Current chain hash
    chain_hash: RwLock<String>,

    /// Event processors
    processors: RwLock<Vec<Arc<dyn AuditProcessor>>>,

    /// Configuration
    config: AuditConfig,
}

/// Audit logger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Enable verbose logging
    pub verbose: bool,

    /// Retention period in days
    pub retention_days: u32,

    /// Maximum entries per file
    pub max_entries_per_file: usize,

    /// Enable real-time alerts
    pub enable_alerts: bool,

    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            retention_days: 90,
            max_entries_per_file: 10000,
            enable_alerts: true,
            alert_thresholds: AlertThresholds::default(),
        }
    }
}

/// Alert thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Failed operations per hour
    pub failed_operations_per_hour: u32,

    /// High-risk operations per day
    pub high_risk_operations_per_day: u32,

    /// Unauthorized access attempts
    pub unauthorized_attempts: u32,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            failed_operations_per_hour: 10,
            high_risk_operations_per_day: 100,
            unauthorized_attempts: 5,
        }
    }
}

/// Audit storage trait
#[async_trait::async_trait]
pub trait AuditStorage: Send + Sync {
    /// Store an audit entry
    async fn store(&self, entry: &AuditEntry) -> Result<()>;

    /// Retrieve entries by criteria
    async fn query(&self, criteria: &QueryCriteria) -> Result<Vec<AuditEntry>>;

    /// Get entry by ID
    async fn get_by_id(&self, id: &str) -> Result<Option<AuditEntry>>;

    /// Delete old entries
    async fn cleanup(&self, older_than: chrono::DateTime<chrono::Utc>) -> Result<u64>;
}

/// Query criteria for audit entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCriteria {
    /// Start time (inclusive)
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,

    /// End time (exclusive)
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,

    /// Event types to include
    pub event_types: Option<Vec<AuditEventType>>,

    /// Actor ID
    pub actor_id: Option<String>,

    /// Resource ID
    pub resource_id: Option<String>,

    /// Result filter
    pub result_filter: Option<ResultFilter>,

    /// Maximum results
    pub limit: Option<usize>,
}

/// Result filter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResultFilter {
    SuccessOnly,
    FailureOnly,
    All,
}

/// Audit event processor trait
#[async_trait::async_trait]
pub trait AuditProcessor: Send + Sync {
    /// Process an audit entry
    async fn process(&self, entry: &AuditEntry) -> Result<()>;
}

/// File-based audit storage
pub struct FileAuditStorage {
    /// Base directory
    base_dir: PathBuf,

    /// Current file
    current_file: Mutex<Option<tokio::fs::File>>,

    /// Entry count in current file
    entry_count: Mutex<usize>,
}

impl FileAuditStorage {
    /// Create new file-based storage
    pub async fn new(base_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&base_dir).await.map_err(|e| {
            RuntimeError::TransactionBuildError(format!("Failed to create audit directory: {}", e))
        })?;

        Ok(Self {
            base_dir,
            current_file: Mutex::new(None),
            entry_count: Mutex::new(0),
        })
    }

    /// Get current file path
    fn current_file_path(&self) -> PathBuf {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        self.base_dir.join(format!("audit_{}.jsonl", timestamp))
    }
}

#[async_trait::async_trait]
impl AuditStorage for FileAuditStorage {
    async fn store(&self, entry: &AuditEntry) -> Result<()> {
        let mut file = self.current_file.lock().await;
        let mut count = self.entry_count.lock().await;

        // Create new file if needed
        if file.is_none() || *count >= 10000 {
            let new_file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(self.current_file_path())
                .await
                .map_err(|e| {
                    RuntimeError::TransactionBuildError(format!(
                        "Failed to create audit file: {}",
                        e
                    ))
                })?;

            *file = Some(new_file);
            *count = 0;
        }

        // Write entry
        if let Some(f) = file.as_mut() {
            let json = serde_json::to_string(entry).map_err(RuntimeError::Serialization)?;

            f.write_all(json.as_bytes()).await.map_err(|e| {
                RuntimeError::TransactionBuildError(format!("Failed to write audit entry: {}", e))
            })?;

            f.write_all(b"\n").await.map_err(|e| {
                RuntimeError::TransactionBuildError(format!("Failed to write newline: {}", e))
            })?;

            *count += 1;
        }

        Ok(())
    }

    async fn query(&self, criteria: &QueryCriteria) -> Result<Vec<AuditEntry>> {
        // Simple implementation - in production, use a proper database
        let mut entries = Vec::new();

        let mut dir = fs::read_dir(&self.base_dir).await.map_err(|e| {
            RuntimeError::TransactionBuildError(format!("Failed to read audit directory: {}", e))
        })?;

        while let Some(entry) = dir.next_entry().await.map_err(|e| {
            RuntimeError::TransactionBuildError(format!("Failed to read directory entry: {}", e))
        })? {
            if entry.path().extension() == Some(std::ffi::OsStr::new("jsonl")) {
                let content = fs::read_to_string(entry.path()).await.map_err(|e| {
                    RuntimeError::TransactionBuildError(format!("Failed to read audit file: {}", e))
                })?;

                for line in content.lines() {
                    if let Ok(entry) = serde_json::from_str::<AuditEntry>(line) {
                        // Apply filters
                        if Self::matches_criteria(&entry, criteria) {
                            entries.push(entry);

                            if let Some(limit) = criteria.limit {
                                if entries.len() >= limit {
                                    return Ok(entries);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(entries)
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<AuditEntry>> {
        let criteria = QueryCriteria {
            start_time: None,
            end_time: None,
            event_types: None,
            actor_id: None,
            resource_id: None,
            result_filter: None,
            limit: Some(1),
        };

        let entries = self.query(&criteria).await?;
        Ok(entries.into_iter().find(|e| e.id == id))
    }

    async fn cleanup(&self, older_than: chrono::DateTime<chrono::Utc>) -> Result<u64> {
        let mut deleted = 0u64;

        let mut dir = fs::read_dir(&self.base_dir).await.map_err(|e| {
            RuntimeError::TransactionBuildError(format!("Failed to read audit directory: {}", e))
        })?;

        while let Some(entry) = dir.next_entry().await.map_err(|e| {
            RuntimeError::TransactionBuildError(format!("Failed to read directory entry: {}", e))
        })? {
            if let Ok(metadata) = entry.metadata().await {
                if let Ok(modified) = metadata.modified() {
                    let modified_time: chrono::DateTime<chrono::Utc> = modified.into();
                    if modified_time < older_than {
                        fs::remove_file(entry.path()).await.map_err(|e| {
                            RuntimeError::TransactionBuildError(format!(
                                "Failed to delete old audit file: {}",
                                e
                            ))
                        })?;
                        deleted += 1;
                    }
                }
            }
        }

        Ok(deleted)
    }
}

impl FileAuditStorage {
    /// Check if entry matches criteria
    fn matches_criteria(entry: &AuditEntry, criteria: &QueryCriteria) -> bool {
        // Time filter
        if let Some(start) = &criteria.start_time {
            if entry.timestamp < *start {
                return false;
            }
        }

        if let Some(end) = &criteria.end_time {
            if entry.timestamp >= *end {
                return false;
            }
        }

        // Event type filter
        if let Some(types) = &criteria.event_types {
            if !types.contains(&entry.event_type) {
                return false;
            }
        }

        // Actor filter
        if let Some(actor_id) = &criteria.actor_id {
            if entry.actor.id != *actor_id {
                return false;
            }
        }

        // Resource filter
        if let Some(resource_id) = &criteria.resource_id {
            if let Some(resource) = &entry.resource {
                if resource.id != *resource_id {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Result filter
        if let Some(filter) = &criteria.result_filter {
            matches!((&entry.result, filter), (OperationResult::Success { .. }, ResultFilter::SuccessOnly) | (OperationResult::Failure { .. }, ResultFilter::FailureOnly) | (_, ResultFilter::All))
        } else {
            true
        }
    }
}

impl AuditLogger {
    /// Create new audit logger
    pub async fn new(storage: Arc<dyn AuditStorage>, config: AuditConfig) -> Result<Self> {
        Ok(Self {
            storage,
            chain_hash: RwLock::new(String::from("genesis")),
            processors: RwLock::new(Vec::new()),
            config,
        })
    }

    /// Add an audit processor
    pub async fn add_processor(&self, processor: Arc<dyn AuditProcessor>) {
        let mut processors = self.processors.write().await;
        processors.push(processor);
    }

    /// Log an audit entry
    pub async fn log(&self, mut entry: AuditEntry) -> Result<()> {
        // Add chain hash
        let previous_hash = self.chain_hash.read().await.clone();
        entry.previous_hash = Some(previous_hash);

        // Calculate entry hash
        entry.entry_hash = self.calculate_hash(&entry);

        // Update chain hash
        *self.chain_hash.write().await = entry.entry_hash.clone();

        // Store entry
        self.storage.store(&entry).await?;

        // Process entry
        let processors = self.processors.read().await;
        for processor in processors.iter() {
            if let Err(e) = processor.process(&entry).await {
                error!("Audit processor error: {}", e);
            }
        }

        if self.config.verbose {
            info!(
                "Audit: {} - {} - {:?}",
                entry.event_type, entry.operation.name, entry.result
            );
        }

        Ok(())
    }

    /// Create audit entry builder
    pub fn entry_builder(&self, event_type: AuditEventType) -> AuditEntryBuilder {
        AuditEntryBuilder::new(event_type)
    }

    /// Query audit entries
    pub async fn query(&self, criteria: &QueryCriteria) -> Result<Vec<AuditEntry>> {
        self.storage.query(criteria).await
    }

    /// Cleanup old entries
    pub async fn cleanup(&self) -> Result<u64> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(self.config.retention_days as i64);
        self.storage.cleanup(cutoff).await
    }

    /// Calculate entry hash
    fn calculate_hash(&self, entry: &AuditEntry) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();

        // Hash key fields
        hasher.update(entry.id.as_bytes());
        hasher.update(entry.timestamp.to_rfc3339().as_bytes());
        hasher.update(format!("{:?}", entry.event_type).as_bytes());
        hasher.update(entry.actor.id.as_bytes());

        if let Some(prev) = &entry.previous_hash {
            hasher.update(prev.as_bytes());
        }

        format!("{:x}", hasher.finalize())
    }
}

/// Audit entry builder
pub struct AuditEntryBuilder {
    entry: AuditEntry,
}

impl AuditEntryBuilder {
    /// Create new builder
    pub fn new(event_type: AuditEventType) -> Self {
        Self {
            entry: AuditEntry {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                event_type,
                actor: Actor {
                    actor_type: ActorType::System,
                    id: "system".to_string(),
                    metadata: HashMap::new(),
                },
                resource: None,
                operation: Operation {
                    name: String::new(),
                    parameters: HashMap::new(),
                    transaction_id: None,
                    signature: None,
                },
                result: OperationResult::Success { details: None },
                context: HashMap::new(),
                parent_id: None,
                previous_hash: None,
                entry_hash: String::new(),
            },
        }
    }

    /// Set actor
    pub fn actor(mut self, actor: Actor) -> Self {
        self.entry.actor = actor;
        self
    }

    /// Set resource
    pub fn resource(mut self, resource: Resource) -> Self {
        self.entry.resource = Some(resource);
        self
    }

    /// Set operation
    pub fn operation(mut self, operation: Operation) -> Self {
        self.entry.operation = operation;
        self
    }

    /// Set result
    pub fn result(mut self, result: OperationResult) -> Self {
        self.entry.result = result;
        self
    }

    /// Add context
    pub fn context(mut self, key: String, value: serde_json::Value) -> Self {
        self.entry.context.insert(key, value);
        self
    }

    /// Set parent ID
    pub fn parent(mut self, parent_id: String) -> Self {
        self.entry.parent_id = Some(parent_id);
        self
    }

    /// Build the entry
    pub fn build(self) -> AuditEntry {
        self.entry
    }
}

/// Alert processor for real-time alerts
pub struct AlertProcessor {
    config: AlertThresholds,
    recent_failures: Mutex<Vec<chrono::DateTime<chrono::Utc>>>,
    recent_high_risk: Mutex<Vec<chrono::DateTime<chrono::Utc>>>,
    unauthorized_attempts: Mutex<HashMap<String, u32>>,
}

impl AlertProcessor {
    pub fn new(config: AlertThresholds) -> Self {
        Self {
            config,
            recent_failures: Mutex::new(Vec::new()),
            recent_high_risk: Mutex::new(Vec::new()),
            unauthorized_attempts: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl AuditProcessor for AlertProcessor {
    async fn process(&self, entry: &AuditEntry) -> Result<()> {
        match &entry.result {
            OperationResult::Failure { error: _, .. } => {
                let mut failures = self.recent_failures.lock().await;
                let now = chrono::Utc::now();

                // Add failure
                failures.push(now);

                // Remove old entries
                let cutoff = now - chrono::Duration::hours(1);
                failures.retain(|t| *t > cutoff);

                // Check threshold
                if failures.len() > self.config.failed_operations_per_hour as usize {
                    warn!(
                        "ALERT: High failure rate detected - {} failures in last hour",
                        failures.len()
                    );
                }
            }
            OperationResult::Success { .. } => {
                // Check for high-risk operations that succeeded
                if entry.event_type == AuditEventType::TransactionSigned {
                    let mut high_risk = self.recent_high_risk.lock().await;
                    high_risk.push(chrono::Utc::now());

                    // Clean old entries (keep only last hour)
                    let cutoff = chrono::Utc::now() - chrono::Duration::hours(1);
                    high_risk.retain(|&timestamp| timestamp > cutoff);
                }

                // Track unauthorized attempts
                if let Some(actor_id) = entry.actor.metadata.get("source_ip") {
                    if entry.operation.name.contains("unauthorized") {
                        let mut attempts = self.unauthorized_attempts.lock().await;
                        *attempts.entry(actor_id.to_string()).or_insert(0) += 1;

                        if let Some(&count) = attempts.get(actor_id) {
                            if count > self.config.unauthorized_attempts {
                                warn!(
                                    "ALERT: Multiple unauthorized attempts from {}: {}",
                                    actor_id, count
                                );
                            }
                        }
                    }
                }
            }
            OperationResult::Partial { .. } => {
                // Partial operations might indicate incomplete or interrupted transactions
                // These could be worth monitoring but don't trigger specific alerts yet
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_type_display() {
        assert_eq!(
            AuditEventType::TransactionConstruction.to_string(),
            "TRANSACTION_CONSTRUCTION"
        );
    }

    #[tokio::test]
    async fn test_audit_entry_builder() {
        let entry = AuditEntryBuilder::new(AuditEventType::TransactionConstruction)
            .actor(Actor {
                actor_type: ActorType::User,
                id: "user123".to_string(),
                metadata: HashMap::new(),
            })
            .operation(Operation {
                name: "create_transaction".to_string(),
                parameters: HashMap::new(),
                transaction_id: None,
                signature: None,
            })
            .build();

        assert_eq!(entry.event_type, AuditEventType::TransactionConstruction);
        assert_eq!(entry.actor.id, "user123");
    }

    #[tokio::test]
    async fn test_file_storage() {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage = FileAuditStorage::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

        let entry = AuditEntryBuilder::new(AuditEventType::SystemEvent)
            .operation(Operation {
                name: "test".to_string(),
                parameters: HashMap::new(),
                transaction_id: None,
                signature: None,
            })
            .build();

        let result = storage.store(&entry).await;
        assert!(result.is_ok());
    }
}
