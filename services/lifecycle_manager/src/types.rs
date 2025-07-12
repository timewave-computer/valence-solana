//! Common types used across the service

use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountRequest {
    pub id: Pubkey,
    pub owner: Pubkey,
    pub capabilities: Vec<String>,
    #[serde(with = "serde_arrays")]
    pub init_state_hash: [u8; 32],
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Pubkey,
    pub owner: Pubkey,
    pub capabilities: Vec<String>,
    #[serde(with = "serde_arrays")]
    pub state_hash: [u8; 32],
    pub is_active: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Pubkey,
    pub owner: Pubkey,
    pub accounts: Vec<Pubkey>,
    pub namespace: String,
    pub is_consumed: bool,
    pub nonce: u64,
    pub created_at: i64,
    pub metadata: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConsumption {
    pub consumed_session: Pubkey,
    pub created_sessions: Vec<Pubkey>,
    #[serde(with = "serde_arrays")]
    pub transaction_signature: [u8; 64],
    pub consumed_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleEvent {
    AccountRequested {
        request_id: Pubkey,
        owner: Pubkey,
        capabilities: Vec<String>,
    },
    AccountInitialized {
        account_id: Pubkey,
        request_id: Pubkey,
    },
    SessionCreated {
        session_id: Pubkey,
        accounts: Vec<Pubkey>,
    },
    SessionConsumed {
        session_id: Pubkey,
        new_sessions: Vec<Pubkey>,
    },
    StateTransition {
        entity_id: Pubkey,
        old_state_hash: [u8; 32],
        new_state_hash: [u8; 32],
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearProgression {
    pub id: String,
    pub current_state: LinearState,
    pub history: Vec<LinearTransition>,
    pub pending_operations: Vec<PendingOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinearState {
    /// Account exists independently
    Account { id: Pubkey },
    
    /// Account is part of a session
    InSession { account_id: Pubkey, session_id: Pubkey },
    
    /// Session is active
    ActiveSession { id: Pubkey },
    
    /// Session has been consumed
    ConsumedSession { id: Pubkey, created_sessions: Vec<Pubkey> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearTransition {
    pub from_state: LinearState,
    pub to_state: LinearState,
    pub timestamp: i64,
    pub transaction_signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingOperation {
    pub operation_type: OperationType,
    pub target: Pubkey,
    pub created_at: i64,
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    InitializeAccount,
    CreateSession,
    ConsumeSession,
    ExecuteBundle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBuilder {
    pub request_id: Pubkey,
    pub capabilities: Vec<String>,
    pub init_state_hash: [u8; 32],
    pub init_state_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionBuilder {
    pub accounts: Vec<Pubkey>,
    pub namespace: String,
    pub suggested_operations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressionRule {
    pub id: String,
    pub name: String,
    pub condition: ProgressionCondition,
    pub action: ProgressionAction,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgressionCondition {
    /// All accounts in session have matching capability
    AllAccountsHaveCapability(String),
    
    /// Session has been idle for duration
    SessionIdleFor(u64),
    
    /// State hash matches pattern
    StateHashMatches([u8; 32]),
    
    /// Custom predicate
    CustomPredicate(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgressionAction {
    /// Consume session and create new ones
    ConsumeAndCreate(Vec<SessionTemplate>),
    
    /// Execute a bundle on the session
    ExecuteBundle(BundleTemplate),
    
    /// Notify external service
    NotifyWebhook(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTemplate {
    pub account_indices: Vec<usize>,
    pub namespace: String,
    pub metadata: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleTemplate {
    pub operations: Vec<OperationTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationTemplate {
    #[serde(with = "serde_arrays")]
    pub function_hash: [u8; 32],
    pub args_template: String,
}

// Custom serde implementation for fixed-size arrays
mod serde_arrays {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    
    pub fn serialize<S, const N: usize>(data: &[u8; N], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        data.serialize(serializer)
    }
    
    pub fn deserialize<'de, D, const N: usize>(deserializer: D) -> Result<[u8; N], D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec = Vec::<u8>::deserialize(deserializer)?;
        let len = vec.len();
        vec.try_into()
            .map_err(|_| serde::de::Error::custom(format!("Expected array of length {}, got {}", N, len)))
    }
}