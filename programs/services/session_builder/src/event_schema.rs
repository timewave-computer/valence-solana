// Event Schema Definition for Off-Chain Service Integration
// This module defines the comprehensive event schema that off-chain services
// must handle for session creation and queue processing.

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Complete event schema for off-chain service integration
/// All events emitted by the session_factory program
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(tag = "event_type")]
pub enum SessionFactoryEvent {
    /// PDA computation for off-chain account creation
    PDAComputed(PDAComputedEvent),
    /// Session reservation for two-phase creation
    SessionReserved(SessionReservedEvent),
    /// Comprehensive session creation with attestation
    SessionCreated(SessionCreatedEvent),
    /// Session status activation
    SessionActivated(SessionActivatedEvent),
    /// Session queued for initialization
    SessionQueued(SessionQueuedEvent),
    /// Session initialized from queue
    SessionInitializedFromQueue(SessionInitializedFromQueueEvent),
    /// Queue processing analytics
    QueueProcessed(QueueProcessedEvent),
    /// Manual session initialization
    ManualSessionInitialized(ManualSessionInitializedEvent),
    /// Emergency session state reset
    EmergencySessionReset(EmergencySessionResetEvent),
}

/// Event emitted for off-chain session creation service
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct PDAComputedEvent {
    /// The computed session PDA where account should be created
    pub expected_pda: Pubkey,
    /// Expected size of the session account
    pub expected_size: usize,
    /// Expected owner (session program)
    pub expected_owner: Pubkey,
    /// Session ID for initialization
    pub session_id: String,
    /// Session owner
    pub owner: Pubkey,
    /// Eval program bound to session
    pub eval_program: Pubkey,
    /// Shard this session is associated with
    pub shard_address: Pubkey,
    /// Namespaces for the session
    pub namespaces: Vec<String>,
    /// Block slot when computed
    pub compute_slot: u64,
}

/// Event emitted when session is reserved
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SessionReservedEvent {
    /// Unique reservation identifier
    pub reservation_id: String,
    /// Reserved session ID
    pub session_id: String,
    /// User who made the reservation
    pub reserved_by: Pubkey,
    /// Intended session owner
    pub session_owner: Pubkey,
    /// Template to use (if any)
    pub template_id: Option<String>,
    /// When the reservation expires
    pub expires_at: i64,
    /// When the reservation was made
    pub reserved_at: i64,
}

/// Comprehensive session creation event with full attestation details
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SessionCreatedEvent {
    /// The session PDA address
    pub session_address: Pubkey,
    /// Session owner
    pub owner: Pubkey,
    /// Eval program bound to session
    pub eval_program: Pubkey,
    /// Shard this session is associated with
    pub shard_address: Pubkey,
    /// Session ID
    pub session_id: String,
    /// Namespaces assigned to session
    pub namespaces: Vec<String>,
    /// Capability that authorized this creation
    pub capability_used: Option<String>,
    /// Verification functions that were executed
    pub verification_functions_executed: Vec<String>,
    /// Full attestation details
    pub attestation: SessionCreationAttestation,
    /// Creation timestamp
    pub created_at: i64,
    /// Account size
    pub account_size: usize,
    /// Block slot when created
    pub creation_slot: u64,
}

/// Session creation attestation embedded in events
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SessionCreationAttestation {
    /// Factory that created this session
    pub factory_address: Pubkey,
    /// Unique attestation ID
    pub attestation_id: String,
    /// Parameters used for creation
    pub creation_parameters: CreationParameters,
    /// Verification signature
    pub verification_signature: Vec<u8>,
    /// Block height when attested
    pub attested_at_height: u64,
}

/// Parameters used for session creation
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct CreationParameters {
    /// Template used (if any)
    pub template_id: Option<String>,
    /// Custom configuration (if used)
    pub custom_config: Option<SessionConfiguration>,
    /// Requested namespaces
    pub requested_namespaces: Vec<String>,
    /// Additional metadata
    pub metadata: Vec<(String, String)>, // key-value pairs
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SessionConfiguration {
    /// Maximum session duration in seconds
    pub max_duration: u64,
    /// Session permissions
    pub permissions: SessionPermissions,
    /// Session-specific settings
    pub settings: SessionSettings,
}

/// Session permissions
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SessionPermissions {
    /// Can read from session
    pub can_read: bool,
    /// Can write to session
    pub can_write: bool,
    /// Can execute instructions
    pub can_execute: bool,
    /// Can manage session state
    pub can_manage: bool,
}

/// Session-specific settings
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SessionSettings {
    /// Auto-expire session after max_duration
    pub auto_expire: bool,
    /// Log all session activity
    pub enable_logging: bool,
    /// Performance monitoring
    pub enable_monitoring: bool,
}

/// Session activated event with comprehensive details
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SessionActivatedEvent {
    /// The session PDA address
    pub session_address: Pubkey,
    /// Session ID
    pub session_id: String,
    /// Session owner
    pub owner: Pubkey,
    /// Eval program bound to session
    pub eval_program: Pubkey,
    /// Shard this session is associated with
    pub shard_address: Pubkey,
    /// Namespaces assigned to session
    pub namespaces: Vec<String>,
    /// When session was activated
    pub activated_at: i64,
    /// Activated by
    pub activated_by: Pubkey,
    /// Block slot when activated
    pub activation_slot: u64,
}

/// Session queued event with comprehensive details
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SessionQueuedEvent {
    /// The session PDA address
    pub session_pda: Pubkey,
    /// Session ID
    pub session_id: String,
    /// Session owner
    pub owner: Pubkey,
    /// Eval program bound to session
    pub eval_program: Pubkey,
    /// Shard this session is associated with
    pub shard_address: Pubkey,
    /// Namespaces assigned to session
    pub namespaces: Vec<String>,
    /// Queued by
    pub queued_by: Pubkey,
    /// Queued at timestamp
    pub queued_at: i64,
    /// Execution deadline
    pub deadline: i64,
    /// Queue position
    pub queue_position: usize,
}

/// Session initialized from queue event with comprehensive details
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SessionInitializedFromQueueEvent {
    /// The session PDA address
    pub session_pda: Pubkey,
    /// Session ID
    pub session_id: String,
    /// Session owner
    pub owner: Pubkey,
    /// Executor
    pub executor: Pubkey,
    /// Queued by
    pub queued_by: Pubkey,
    /// When session was initialized
    pub initialized_at: i64,
    /// Queue position
    pub queue_position: usize,
}

/// Queue processed event for analytics and monitoring
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct QueueProcessedEvent {
    /// Queue processor (executor)
    pub processor: Pubkey,
    /// Number of sessions processed in this batch
    pub sessions_processed: u32,
    /// Number of failed initializations
    pub failed_initializations: u32,
    /// Total queue size before processing
    pub queue_size_before: usize,
    /// Total queue size after processing
    pub queue_size_after: usize,
    /// Processing duration (approximate)
    pub processing_duration_ms: u64,
    /// Timestamp when processing started
    pub processing_started_at: i64,
    /// Timestamp when processing completed
    pub processing_completed_at: i64,
    /// Total sessions processed by this queue (cumulative)
    pub total_processed: u64,
    /// Total failed sessions by this queue (cumulative)
    pub total_failed: u64,
}

/// Manual session initialization event with comprehensive details
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ManualSessionInitializedEvent {
    /// The session PDA address
    pub session_pda: Pubkey,
    /// Session ID
    pub session_id: String,
    /// Session owner
    pub owner: Pubkey,
    /// Authority
    pub authority: Pubkey,
    /// Force activation
    pub force_activation: bool,
    /// When session was initialized
    pub initialized_at: i64,
}

/// Emergency session reset event with comprehensive details
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct EmergencySessionResetEvent {
    /// The session PDA address
    pub session_pda: Pubkey,
    /// Session ID
    pub session_id: String,
    /// Session owner
    pub owner: Pubkey,
    /// Authority
    pub authority: Pubkey,
    /// Old session status
    pub old_status: SessionStatus,
    /// New session status
    pub new_status: SessionStatus,
    /// Old optimistic state
    pub old_optimistic_state: SessionState,
    /// New optimistic state
    pub new_optimistic_state: SessionState,
    /// When session was reset
    pub reset_at: i64,
}

/// Session status tracking
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum SessionStatus {
    /// Session creation requested but account not yet created
    Requested,
    /// Account created but not yet initialized
    Created,
    /// Session fully initialized and active
    Active,
    /// Session has been closed/deactivated
    Closed,
}

/// Optimistic session state for UX improvement
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum SessionState {
    /// Session is pending activation (optimistic)
    Pending,
    /// Session is fully active and initialized
    Active,
    /// Session has been closed
    Closed,
}

/// Required service behavior specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceBehaviorSpec {
    /// Events that must be monitored
    pub required_event_subscriptions: Vec<String>,
    /// Maximum processing delay for each event type (milliseconds)
    pub max_processing_delays: std::collections::BTreeMap<String, u64>,
    /// Retry configuration requirements
    pub retry_requirements: RetryRequirements,
    /// Health check requirements
    pub health_check_requirements: HealthCheckRequirements,
    /// Metrics that must be tracked
    pub required_metrics: Vec<String>,
}

/// Retry configuration requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryRequirements {
    /// Maximum number of retries per operation
    pub max_retries: u32,
    /// Initial delay between retries (milliseconds)
    pub initial_delay_ms: u64,
    /// Maximum delay between retries (milliseconds)
    pub max_delay_ms: u64,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
}

/// Health check requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckRequirements {
    /// Health check interval (seconds)
    pub interval_secs: u64,
    /// Required health check endpoints
    pub required_endpoints: Vec<String>,
    /// Maximum response time for health checks (milliseconds)
    pub max_response_time_ms: u64,
}

impl ServiceBehaviorSpec {
    /// Get the standard service behavior specification
    pub fn standard() -> Self {
        let mut max_processing_delays = std::collections::BTreeMap::new();
        max_processing_delays.insert("PDAComputed".to_string(), 5000); // 5 seconds
        max_processing_delays.insert("SessionQueued".to_string(), 1000); // 1 second
        max_processing_delays.insert("SessionReserved".to_string(), 2000); // 2 seconds
        
        Self {
            required_event_subscriptions: vec![
                "PDAComputed".to_string(),
                "SessionQueued".to_string(),
                "SessionReserved".to_string(),
                "QueueProcessed".to_string(),
            ],
            max_processing_delays,
            retry_requirements: RetryRequirements {
                max_retries: 5,
                initial_delay_ms: 100,
                max_delay_ms: 30000,
                backoff_multiplier: 2.0,
            },
            health_check_requirements: HealthCheckRequirements {
                interval_secs: 30,
                required_endpoints: vec![
                    "/health".to_string(),
                    "/metrics".to_string(),
                    "/stats".to_string(),
                ],
                max_response_time_ms: 1000,
            },
            required_metrics: vec![
                "accounts_created_total".to_string(),
                "accounts_failed_total".to_string(),
                "event_processing_duration".to_string(),
                "rpc_connection_status".to_string(),
                "service_uptime_seconds".to_string(),
            ],
        }
    }
} 