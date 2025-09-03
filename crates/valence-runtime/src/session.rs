//! Session management and operations

use crate::Result;
use serde::{Deserialize, Serialize};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::{debug, info};

use valence_kernel::{
    state::{Session, GuardAccount},
    namespace::Namespace,
};

// ================================
// Operation Types
// ================================

/// Session operation request for batch building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionOperationRequest {
    pub session_pubkey: Pubkey,
    pub operations: Vec<KernelOperationRequest>,
    pub compute_budget: Option<u32>,
    pub priority_fee: Option<u64>,
}

/// Individual operation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelOperationRequest {
    pub operation_type: String,
    pub accounts: Vec<AccountRequest>,
    pub data: Vec<u8>,
    pub expected_compute: Option<u64>,
}

/// Account requirement for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountRequest {
    pub pubkey: Pubkey,
    pub access_mode: u8,
    pub account_type: AccountType,
}

/// Account type classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountType {
    Token { mint: Pubkey },
    TokenAccount { mint: Pubkey, owner: Pubkey },
    Program { executable: bool },
    Data { discriminator: [u8; 8] },
    Session,
    Guard,
    Namespace,
    ChildAccount,
}

/// Operation execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub session: Pubkey,
    pub operation_index: usize,
    pub success: bool,
    pub compute_units_used: u64,
    pub logs: Vec<String>,
    pub error: Option<String>,
}

// ================================
// Session State Types
// ================================

/// Cached session state with kernel integration
#[derive(Debug, Clone)]
pub struct SessionState {
    /// Session account pubkey
    pub session_pubkey: Pubkey,
    
    /// On-chain session data
    pub session_data: Session,
    
    /// Associated guard account
    pub guard_pubkey: Option<Pubkey>,
    
    /// Account lookup table (ALT) pubkey
    pub alt_pubkey: Option<Pubkey>,
    
    /// Last sync slot to avoid redundant fetches
    pub last_sync_slot: u64,
    
    /// Off-chain metrics tracking
    pub metrics: SessionMetrics,
    
    /// Cached namespace information
    pub namespace_info: Option<NamespaceInfo>,
}

/// Guard account state cache
#[derive(Clone)]
pub struct GuardState {
    pub guard_pubkey: Pubkey,
    pub guard_data: GuardAccount,
    pub last_sync_slot: u64,
}

/// Namespace state cache
#[derive(Debug, Clone)]
pub struct NamespaceState {
    pub namespace_pubkey: Pubkey,
    pub namespace_data: Namespace,
    pub last_sync_slot: u64,
}

/// Namespace information for session operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceInfo {
    pub path: String,
    pub depth: u8,
    pub parent: Option<Pubkey>,
    pub children: Vec<Pubkey>,
    pub access_permissions: Vec<String>,
}

/// Session execution metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub total_compute_units: u64,
    pub average_compute_per_operation: u64,
    pub total_accounts_borrowed: u64,
    pub peak_concurrent_borrows: u64,
    pub guard_evaluations: u64,
    pub guard_failures: u64,
    pub namespace_operations: u64,
}

impl SessionMetrics {
    /// Update metrics with operation result
    pub fn record_operation(&mut self, success: bool, compute_units: u64) {
        self.total_operations += 1;
        if success {
            self.successful_operations += 1;
        } else {
            self.failed_operations += 1;
        }
        self.total_compute_units += compute_units;
        self.average_compute_per_operation = 
            self.total_compute_units / self.total_operations.max(1);
    }
    
    /// Update borrow metrics
    pub fn record_borrow(&mut self, current_borrows: u64) {
        self.total_accounts_borrowed += 1;
        self.peak_concurrent_borrows = self.peak_concurrent_borrows.max(current_borrows);
    }

    /// Record guard evaluation
    pub fn record_guard_evaluation(&mut self, success: bool) {
        self.guard_evaluations += 1;
        if !success {
            self.guard_failures += 1;
        }
    }

    /// Record namespace operation
    pub fn record_namespace_operation(&mut self) {
        self.namespace_operations += 1;
    }
}

impl SessionState {
    /// Create SessionState from account data
    pub fn from_account_data(data: &[u8]) -> crate::Result<Self> {
        // This is a simplified implementation for demonstration
        // In production, this would deserialize the actual account data
        if data.len() < 32 {
            return Err(crate::RuntimeError::InvalidAccountData);
        }
        
        // Create a dummy session with minimal required fields
        let session_pubkey = Pubkey::new_unique();
        let session_data = Session {
            namespace: valence_kernel::namespace::NamespacePath::new("default").unwrap_or_else(|_| {
                // Create empty namespace path as fallback
                valence_kernel::namespace::NamespacePath { path: [0u8; 256], len: 0 }
            }),
            guard_account: Pubkey::new_unique(),
            account_lookup: Pubkey::new_unique(),
            owner: session_pubkey,
            shard: Pubkey::new_unique(),
            parent_session: None,
            usage_count: 0,
            metadata: [0u8; 32],
            created_at: 0,
            updated_at: 0,
            borrowed_accounts: Default::default(),
            borrowed_bitmap: 0,
            cpi_depth: 0,
            active: true,
            nonce: 0,
            child_accounts: [Pubkey::default(); 8],
            child_count: 0,
            child_sessions: [Pubkey::default(); 8],
            child_session_count: 0,
        };
        
        Ok(SessionState {
            session_pubkey,
            session_data,
            guard_pubkey: None,
            alt_pubkey: None,
            last_sync_slot: 0,
            metrics: SessionMetrics::default(),
            namespace_info: None,
        })
    }
}

// ================================
// Session Manager
// ================================

/// Session manager for runtime
pub struct SessionManager {
    rpc_client: Arc<RpcClient>,
    sessions: Arc<RwLock<HashMap<Pubkey, SessionCache>>>,
    metrics: Arc<RwLock<SessionManagerMetrics>>,
}

/// Cached session data
#[derive(Debug, Clone)]
struct SessionCache {
    state: SessionState,
    last_updated: chrono::DateTime<chrono::Utc>,
    ttl_seconds: u64,
}

/// Session manager metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionManagerMetrics {
    pub total_sessions: u32,
    pub active_sessions: u32,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub load_errors: u32,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(rpc_client: Arc<RpcClient>) -> Self {
        Self {
            rpc_client,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(SessionManagerMetrics::default())),
        }
    }

    /// Load session state from on-chain account
    pub async fn load_session(&self, session_pubkey: Pubkey) -> Result<SessionState> {
        info!("Loading session: {}", session_pubkey);

        // Check cache first
        {
            let sessions = self.sessions.read().await;
            if let Some(cached) = sessions.get(&session_pubkey) {
                if !self.is_cache_expired(&cached) {
                    debug!("Cache hit for session: {}", session_pubkey);
                    self.metrics.write().await.cache_hits += 1;
                    return Ok(cached.state.clone());
                }
            }
        }

        // Cache miss - load from chain
        debug!("Cache miss for session: {}", session_pubkey);
        self.metrics.write().await.cache_misses += 1;

        let account = match self.rpc_client.get_account(&session_pubkey).await {
            Ok(account) => account,
            Err(e) => {
                self.metrics.write().await.load_errors += 1;
                return Err(e.into());
            }
        };

        // Parse session state (simplified)
        let state = SessionState::from_account_data(&account.data)?;

        // Update cache
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_pubkey, SessionCache {
                state: state.clone(),
                last_updated: chrono::Utc::now(),
                ttl_seconds: 300, // 5 minutes
            });
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            let sessions = self.sessions.read().await;
            metrics.total_sessions = sessions.len() as u32;
            metrics.active_sessions += 1;
        }

        Ok(state)
    }

    /// Get cached session if available
    pub async fn get_cached_session(&self, session_pubkey: &Pubkey) -> Option<SessionState> {
        let sessions = self.sessions.read().await;
        if let Some(cached) = sessions.get(session_pubkey) {
            if !self.is_cache_expired(cached) {
                return Some(cached.state.clone());
            }
        }
        None
    }

    /// Invalidate cached session
    pub async fn invalidate_session(&self, session_pubkey: &Pubkey) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_pubkey);
        debug!("Invalidated cached session: {}", session_pubkey);
    }

    /// Clear all cached sessions
    pub async fn clear_cache(&self) {
        let mut sessions = self.sessions.write().await;
        sessions.clear();
        info!("Cleared all cached sessions");
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> SessionManagerMetrics {
        self.metrics.read().await.clone()
    }

    /// Check if cached session is expired
    fn is_cache_expired(&self, cached: &SessionCache) -> bool {
        let now = chrono::Utc::now();
        let elapsed = now.signed_duration_since(cached.last_updated);
        elapsed.num_seconds() as u64 > cached.ttl_seconds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_manager_creation() {
        let rpc_client = Arc::new(RpcClient::new("https://api.mainnet-beta.solana.com".to_string()));
        let manager = SessionManager::new(rpc_client);
        
        let metrics = manager.get_metrics().await;
        assert_eq!(metrics.total_sessions, 0);
        assert_eq!(metrics.cache_hits, 0);
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let rpc_client = Arc::new(RpcClient::new("https://api.mainnet-beta.solana.com".to_string()));
        let manager = SessionManager::new(rpc_client);
        
        let session_pubkey = Pubkey::new_unique();
        
        // Should return None for non-existent session
        let cached = manager.get_cached_session(&session_pubkey).await;
        assert!(cached.is_none());
        
        // Test cache invalidation
        manager.invalidate_session(&session_pubkey).await;
        
        // Test cache clearing
        manager.clear_cache().await;
        
        let metrics = manager.get_metrics().await;
        assert_eq!(metrics.total_sessions, 0);
    }
}