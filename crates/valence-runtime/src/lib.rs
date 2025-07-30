//! Valence Runtime Service
//!
//! Off-chain service for monitoring on-chain state and orchestrating protocol flows.
//! This runtime does not hold keys but builds transactions for external signing.

// ================================
// Module Declarations
// ================================

pub mod core;
pub mod types;

// Core runtime functionality
// Session management
pub mod session;
pub use session::{
    SessionManager, SessionState, SessionMetrics,
    SessionOperationRequest, KernelOperationRequest, AccountRequest, 
    AccountType, OperationResult,
};

// Transaction building and management
pub mod transaction {
    pub mod builder;
    pub mod instructions;
    
    pub use builder::{TransactionBuilder, UnsignedTransaction, TransactionMetadata, SimulationResult};
}

// State monitoring and event streaming
pub mod monitoring {
    pub mod state_monitor;
    pub mod event_stream;
    
    pub use state_monitor::{StateMonitor, StateUpdate};
    pub use event_stream::{EventStream, Event};
}

// Flow coordination and execution
pub mod coordination;
pub use coordination::{Coordinator, ProtocolFlow};

// Security utilities and validation
pub mod security;

// ================================
// Public API Re-exports
// ================================

// Configuration and errors
pub use core::{RuntimeConfig, RuntimeError, Result};

// Session management (re-exported above)

// Transaction management  
pub use transaction::{TransactionBuilder, UnsignedTransaction, TransactionMetadata};

// Monitoring and events
pub use monitoring::{StateMonitor, StateUpdate, EventStream, Event};

// Coordination (re-exported above)

// Security
pub use security::{AuditLogger, SecurityAnalyzer, SecurityContext, TransactionValidator};
pub use security::{CompositeSigningService, SigningRequest, SigningResponse, SigningService};

// Common types
pub use types::{
    RuntimeSessionParams, RuntimeMetrics, RuntimeEvent, RuntimeConfiguration,
    KernelExecutionPlan, SessionHealth, GuardHealth, SessionStatus,
};

// ================================
// Main Runtime Implementation
// ================================

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Parameters for creating a child account
#[derive(Debug, Clone)]
pub struct CreateChildAccountParams {
    pub payer: Pubkey,
    pub session: Pubkey,
    pub child_account: Pubkey,
    pub namespace_suffix: String,
    pub initial_lamports: u64,
    pub space: u64,
    pub owner_program: Pubkey,
}

/// Main runtime service compatible with valence-kernel
pub struct Runtime {
    config: RuntimeConfig,
    rpc_client: Arc<RpcClient>,
    state_monitor: Arc<RwLock<StateMonitor>>,
    coordinator: Arc<Coordinator>,
    event_stream: Arc<EventStream>,
    signing_service: Arc<CompositeSigningService>,
    #[allow(dead_code)]
    audit_logger: Arc<AuditLogger>,
    transaction_validator: Arc<TransactionValidator>,
    session_manager: Arc<SessionManager>,
    runtime_metrics: Arc<RwLock<RuntimeMetrics>>,
}

impl Runtime {
    /// Create a new runtime instance
    pub async fn new(config: RuntimeConfig) -> Result<Self> {
        info!("Initializing Valence runtime");

        let rpc_client = Arc::new(RpcClient::new_with_commitment(
            config.rpc_url.clone(),
            config.commitment,
        ));

        let event_stream = Arc::new(EventStream::new());

        let state_monitor = Arc::new(RwLock::new(
            StateMonitor::new(config.ws_url.clone(), event_stream.clone()).await?,
        ));

        let coordinator = Arc::new(Coordinator::new(rpc_client.clone(), event_stream.clone()));

        // Initialize security components
        let security_context = SecurityContext {
            timestamp: chrono::Utc::now(),
            environment: security::Environment::Production,
            policies: security::SecurityPolicies::default(),
            session: None,
        };

        // Initialize signing service
        let signing_service = Arc::new(CompositeSigningService::new(
            security::signing::SigningBackend::LocalKeypair,
        ));

        // Initialize audit logger
        let audit_storage = Arc::new(
            security::audit::FileAuditStorage::new(std::path::PathBuf::from("./audit_logs"))
                .await?,
        );
        let audit_logger = Arc::new(
            AuditLogger::new(audit_storage, security::audit::AuditConfig::default()).await?,
        );

        // Initialize transaction validator
        let transaction_validator = Arc::new(TransactionValidator::new(
            rpc_client.clone(),
            security_context,
        ));

        // Initialize session manager for kernel compatibility
        let session_manager = Arc::new(SessionManager::new(rpc_client.clone()));

        // Initialize runtime metrics
        let runtime_metrics = Arc::new(RwLock::new(RuntimeMetrics::default()));

        Ok(Self {
            config,
            rpc_client,
            state_monitor,
            coordinator,
            event_stream,
            signing_service,
            audit_logger,
            transaction_validator,
            session_manager,
            runtime_metrics,
        })
    }

    /// Start the runtime service
    pub async fn start(&self) -> Result<()> {
        info!("Starting Valence runtime service");

        // Start state monitoring
        let monitor = self.state_monitor.read().await;
        monitor.start().await?;

        // Start coordinator
        self.coordinator.start().await?;

        info!("Runtime service started successfully");
        Ok(())
    }

    /// Stop the runtime service
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Valence runtime service");

        // Stop coordinator
        self.coordinator.stop().await?;

        // Stop state monitoring
        let monitor = self.state_monitor.read().await;
        monitor.stop().await?;

        info!("Runtime service stopped");
        Ok(())
    }

    /// Subscribe to runtime events
    pub async fn subscribe(&self) -> tokio::sync::broadcast::Receiver<Event> {
        self.event_stream.subscribe().await
    }

    /// Get the RPC client
    pub fn rpc_client(&self) -> &Arc<RpcClient> {
        &self.rpc_client
    }

    /// Get the transaction builder
    pub fn transaction_builder(&self) -> TransactionBuilder {
        TransactionBuilder::new(self.rpc_client.clone())
    }

    /// Generate deterministic account addresses
    pub fn derive_account_address(program_id: &Pubkey, seeds: &[&[u8]]) -> (Pubkey, u8) {
        Pubkey::find_program_address(seeds, program_id)
    }

    /// Get the runtime configuration
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Get the signing service
    pub fn signing_service(&self) -> &Arc<CompositeSigningService> {
        &self.signing_service
    }

    /// Get the transaction validator
    pub fn transaction_validator(&self) -> &Arc<TransactionValidator> {
        &self.transaction_validator
    }

    // Session management methods
    pub fn session_manager(&self) -> &Arc<SessionManager> {
        &self.session_manager
    }

    pub async fn load_session(&self, session_pubkey: Pubkey) -> Result<SessionState> {
        let state = self.session_manager.load_session(session_pubkey).await?;
        
        // Emit event using existing SessionCreationRequested variant
        self.event_stream.emit(Event::SessionCreationRequested {
            shard: session_pubkey, // Using session as shard for now
            namespace: "default".to_string(),
            owner: session_pubkey, // Using session as owner for now
        }).await;

        // Update metrics
        let mut metrics = self.runtime_metrics.write().await;
        metrics.active_sessions += 1;

        Ok(state)
    }

    // Additional runtime methods would go here...
}