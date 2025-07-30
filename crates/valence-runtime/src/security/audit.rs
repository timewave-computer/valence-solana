//! Audit logging and compliance tracking

use crate::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tracing::error;

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    TransactionSigned,
    TransactionValidated,
    SessionCreated,
    SecurityViolation,
    ConfigurationChanged,
    AuthenticationAttempt,
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditEventType::TransactionSigned => write!(f, "transaction_signed"),
            AuditEventType::TransactionValidated => write!(f, "transaction_validated"),
            AuditEventType::SessionCreated => write!(f, "session_created"),
            AuditEventType::SecurityViolation => write!(f, "security_violation"),
            AuditEventType::ConfigurationChanged => write!(f, "configuration_changed"),
            AuditEventType::AuthenticationAttempt => write!(f, "authentication_attempt"),
        }
    }
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: AuditEventType,
    pub actor: Option<String>,
    pub resource: Option<String>,
    pub outcome: AuditOutcome,
    pub details: HashMap<String, serde_json::Value>,
    pub session_id: Option<String>,
}

/// Audit outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditOutcome {
    Success,
    Failure,
    Denied,
    Error(String),
}

impl AuditEntry {
    /// Create a new audit entry builder
    pub fn builder(event_type: AuditEventType) -> AuditEntryBuilder {
        AuditEntryBuilder::new(event_type)
    }

    /// Create a simple success entry
    pub fn success(event_type: AuditEventType, actor: String) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            event_type,
            actor: Some(actor),
            resource: None,
            outcome: AuditOutcome::Success,
            details: HashMap::new(),
            session_id: None,
        }
    }

    /// Create a simple failure entry
    pub fn failure(event_type: AuditEventType, actor: String, error: String) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            event_type,
            actor: Some(actor),
            resource: None,
            outcome: AuditOutcome::Error(error),
            details: HashMap::new(),
            session_id: None,
        }
    }
}

/// Builder for audit entries
pub struct AuditEntryBuilder {
    entry: AuditEntry,
}

impl AuditEntryBuilder {
    pub fn new(event_type: AuditEventType) -> Self {
        Self {
            entry: AuditEntry {
                timestamp: chrono::Utc::now(),
                event_type,
                actor: None,
                resource: None,
                outcome: AuditOutcome::Success,
                details: HashMap::new(),
                session_id: None,
            },
        }
    }

    pub fn actor(mut self, actor: String) -> Self { self.entry.actor = Some(actor); self }
    pub fn resource(mut self, resource: String) -> Self { self.entry.resource = Some(resource); self }
    pub fn outcome(mut self, outcome: AuditOutcome) -> Self { self.entry.outcome = outcome; self }
    pub fn session_id(mut self, session_id: String) -> Self { self.entry.session_id = Some(session_id); self }
    
    pub fn detail<V: Into<serde_json::Value>>(mut self, key: String, value: V) -> Self {
        self.entry.details.insert(key, value.into());
        self
    }

    pub fn build(self) -> AuditEntry { self.entry }
}

/// Audit storage trait
#[async_trait]
pub trait AuditStorage: Send + Sync {
    async fn store(&self, entry: &AuditEntry) -> Result<()>;
    async fn query(&self, filter: &AuditFilter) -> Result<Vec<AuditEntry>>;
}

/// Audit query filter
#[derive(Debug, Clone)]
pub struct AuditFilter {
    pub event_types: Option<Vec<AuditEventType>>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub actor: Option<String>,
    pub limit: Option<usize>,
}

impl Default for AuditFilter {
    fn default() -> Self {
        Self {
            event_types: None,
            start_time: None,
            end_time: None,
            actor: None,
            limit: Some(1000),
        }
    }
}

/// Audit configuration
#[derive(Debug, Clone)]
pub struct AuditConfig {
    pub enabled: bool,
    pub retention_days: u32,
    pub max_entries_per_file: usize,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_days: 90,
            max_entries_per_file: 10000,
        }
    }
}

/// File-based audit storage
pub struct FileAuditStorage {
    directory: PathBuf,
    current_file: Arc<tokio::sync::Mutex<Option<File>>>,
}

impl FileAuditStorage {
    pub async fn new(directory: PathBuf) -> Result<Self> {
        tokio::fs::create_dir_all(&directory).await?;
        Ok(Self {
            directory,
            current_file: Arc::new(tokio::sync::Mutex::new(None)),
        })
    }

    async fn get_current_file(&self) -> Result<File> {
        let mut current_file = self.current_file.lock().await;
        
        if current_file.is_none() {
            let filename = format!("audit_{}.jsonl", chrono::Utc::now().format("%Y%m%d"));
            let filepath = self.directory.join(filename);
            
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(filepath)
                .await?;
            
            *current_file = Some(file);
        }
        
        // This is a simplified approach - in production you'd want to handle file rotation
        Ok(OpenOptions::new().create(true).append(true).open(
            self.directory.join(format!("audit_{}.jsonl", chrono::Utc::now().format("%Y%m%d")))
        ).await?)
    }
}

#[async_trait]
impl AuditStorage for FileAuditStorage {
    async fn store(&self, entry: &AuditEntry) -> Result<()> {
        let mut file = self.get_current_file().await?;
        let json_line = serde_json::to_string(entry)?;
        file.write_all(format!("{}\n", json_line).as_bytes()).await?;
        file.flush().await?;
        Ok(())
    }

    async fn query(&self, _filter: &AuditFilter) -> Result<Vec<AuditEntry>> {
        // Simplified implementation - would need proper file reading and filtering
        Ok(Vec::new())
    }
}

/// Main audit logger
pub struct AuditLogger {
    storage: Arc<dyn AuditStorage>,
    config: AuditConfig,
}

impl AuditLogger {
    pub async fn new(storage: Arc<dyn AuditStorage>, config: AuditConfig) -> Result<Self> {
        Ok(Self { storage, config })
    }

    /// Log an audit entry
    pub async fn log(&self, entry: AuditEntry) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        if let Err(e) = self.storage.store(&entry).await {
            error!("Failed to store audit entry: {}", e);
            return Err(e);
        }

        Ok(())
    }

    /// Log a transaction signing event
    pub async fn log_transaction_signed(&self, actor: String, transaction_hash: String) -> Result<()> {
        let entry = AuditEntry::builder(AuditEventType::TransactionSigned)
            .actor(actor)
            .resource(transaction_hash.clone())
            .detail("transaction_hash".to_string(), transaction_hash)
            .build();
        
        self.log(entry).await
    }

    /// Log a security violation
    pub async fn log_security_violation(&self, actor: String, violation: String) -> Result<()> {
        let entry = AuditEntry::builder(AuditEventType::SecurityViolation)
            .actor(actor)
            .outcome(AuditOutcome::Denied)
            .detail("violation".to_string(), violation)
            .build();
        
        self.log(entry).await
    }

    /// Query audit logs
    pub async fn query(&self, filter: AuditFilter) -> Result<Vec<AuditEntry>> {
        self.storage.query(&filter).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_audit_event_type_display() {
        assert_eq!(AuditEventType::TransactionSigned.to_string(), "transaction_signed");
        assert_eq!(AuditEventType::SecurityViolation.to_string(), "security_violation");
    }

    #[test]
    fn test_audit_entry_builder() {
        let entry = AuditEntry::builder(AuditEventType::TransactionSigned)
            .actor("test_user".to_string())
            .resource("tx_123".to_string())
            .detail("amount".to_string(), 1000)
            .build();

        assert_eq!(entry.event_type.to_string(), "transaction_signed");
        assert_eq!(entry.actor, Some("test_user".to_string()));
        assert_eq!(entry.resource, Some("tx_123".to_string()));
        assert!(entry.details.contains_key("amount"));
    }

    #[tokio::test]
    async fn test_file_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileAuditStorage::new(temp_dir.path().to_path_buf()).await.unwrap();
        
        let entry = AuditEntry::success(AuditEventType::TransactionSigned, "test_user".to_string());
        assert!(storage.store(&entry).await.is_ok());
    }
}