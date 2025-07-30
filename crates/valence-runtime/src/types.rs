//! Common types for valence-runtime

use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

// Re-export kernel types for runtime use
pub use valence_kernel::{
    state::{Session, SessionAccountLookup, GuardAccount},
    namespace::{Namespace, NamespacePath},
    KernelOperation, OperationBatch,
    ACCESS_MODE_READ, ACCESS_MODE_WRITE,
    errors::KernelError,
};

/// Session creation parameters for runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSessionParams {
    pub namespace_path: String,
    pub max_compute_units: u32,
    pub auto_sync: bool,
    pub cache_ttl_seconds: u64,
    pub enable_metrics: bool,
}

impl Default for RuntimeSessionParams {
    fn default() -> Self {
        Self {
            namespace_path: "default".to_string(),
            max_compute_units: 200_000,
            auto_sync: true,
            cache_ttl_seconds: 300,
            enable_metrics: true,
        }
    }
}

/// Runtime metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeMetrics {
    pub active_sessions: u32,
    pub pending_transactions: u32,
    pub total_operations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub error_count: u32,
}

/// Runtime event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeEvent {
    SessionCreated { session: Pubkey, namespace: String },
    TransactionBuilt { description: String, signers: Vec<Pubkey> },
    ValidationCompleted { success: bool, errors: u32 },
    SecurityViolation { actor: String, violation: String },
}

/// Runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfiguration {
    pub rpc_url: String,
    pub ws_url: String,
    pub commitment: String,
    pub retry_attempts: u32,
    pub timeout_seconds: u64,
    pub enable_validation: bool,
    pub enable_audit_logging: bool,
}

impl Default for RuntimeConfiguration {
    fn default() -> Self {
        Self {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            ws_url: "wss://api.mainnet-beta.solana.com".to_string(),
            commitment: "confirmed".to_string(),
            retry_attempts: 3,
            timeout_seconds: 30,
            enable_validation: true,
            enable_audit_logging: true,
        }
    }
}

/// Kernel execution plan
#[derive(Debug, Clone)]
pub struct KernelExecutionPlan {
    pub operations: Vec<KernelOperation>,
    pub accounts: Vec<Pubkey>,
    pub estimated_compute: u64,
    pub requires_alt: bool,
}

/// Session health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionHealth {
    Healthy,
    Warning(String),
    Critical(String),
    Unavailable,
}

/// Guard health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardHealth {
    pub guard_pubkey: Pubkey,
    pub status: SessionHealth,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub error_count: u32,
}

/// Session status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Inactive,
    Suspended,
    Expired,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_configuration_default() {
        let config = RuntimeConfiguration::default();
        assert_eq!(config.retry_attempts, 3);
        assert_eq!(config.commitment, "confirmed");
        assert!(config.enable_validation);
    }

    #[test]
    fn test_runtime_metrics() {
        let mut metrics = RuntimeMetrics::default();
        metrics.active_sessions = 5;
        metrics.total_operations = 100;
        
        assert_eq!(metrics.active_sessions, 5);
        assert_eq!(metrics.total_operations, 100);
    }

    #[test]
    fn test_guard_check_types() {
        let health = GuardHealth {
            guard_pubkey: Pubkey::new_unique(),
            status: SessionHealth::Healthy,
            last_check: chrono::Utc::now(),
            error_count: 0,
        };

        assert_eq!(health.status, SessionHealth::Healthy);
        assert_eq!(health.error_count, 0);
    }

    #[test]
    fn test_session_health_serialization() {
        let health = SessionHealth::Warning("Test warning".to_string());
        let serialized = serde_json::to_string(&health).unwrap();
        let deserialized: SessionHealth = serde_json::from_str(&serialized).unwrap();
        
        match deserialized {
            SessionHealth::Warning(msg) => assert_eq!(msg, "Test warning"),
            _ => panic!("Expected Warning variant"),
        }
    }
}