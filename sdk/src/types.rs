//! Common types used across Valence SDK

use anchor_lang::prelude::*;
use serde::{Deserialize, Serialize};

// Re-export types from shard program for off-chain use
// These need serde derives for off-chain usage

/// Account request (awaiting initialization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountRequest {
    pub id: Pubkey,
    pub owner: Pubkey,
    pub capabilities: Vec<String>,
    #[serde(with = "serde_arrays")]
    pub init_state_hash: [u8; 32],
    pub created_at: i64,
}

/// Account with capabilities
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

/// Session - Linear type containing multiple accounts
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

/// Session consumption record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConsumption {
    pub consumed_session: Pubkey,
    pub created_sessions: Vec<Pubkey>,
    #[serde(with = "serde_arrays")]
    pub transaction_signature: [u8; 64],
    pub consumed_at: i64,
}

/// Bundle definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    pub operations: Vec<Operation>,
    pub mode: ExecutionMode,
    pub session: Pubkey,
}

/// Single operation in a bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    #[serde(with = "serde_arrays")]
    pub function_hash: [u8; 32],
    pub args: Vec<u8>,
    #[serde(with = "option_serde_arrays")]
    pub expected_diff: Option<[u8; 32]>,
    pub target_account: Option<Pubkey>,
}

/// Bundle execution mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionMode {
    Sync,
    Async,
}

/// Execution state for async bundles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionState {
    pub bundle_id: Pubkey,
    pub current_operation: u16,
    pub total_operations: u16,
    #[serde(with = "serde_arrays")]
    pub state_hash: [u8; 32],
    pub is_complete: bool,
    pub operations: Vec<Operation>,
    pub session: Pubkey,
}

// Custom serde implementation for fixed-size arrays
mod serde_arrays {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    
    pub fn serialize<S, T, const N: usize>(data: &[T; N], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        data.serialize(serializer)
    }
    
    pub fn deserialize<'de, D, T, const N: usize>(deserializer: D) -> Result<[T; N], D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de> + Default + Copy,
    {
        let vec = Vec::<T>::deserialize(deserializer)?;
        let len = vec.len();
        vec.try_into()
            .map_err(|_| serde::de::Error::custom(format!("Expected array of length {}, got {}", N, len)))
    }
}

mod option_serde_arrays {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    
    pub fn serialize<S, T, const N: usize>(data: &Option<[T; N]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
        [T; N]: Serialize,
    {
        data.serialize(serializer)
    }
    
    pub fn deserialize<'de, D, T, const N: usize>(deserializer: D) -> Result<Option<[T; N]>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de> + Default + Copy,
    {
        Option::<Vec<T>>::deserialize(deserializer)?
            .map(|vec| {
                let len = vec.len();
                vec.try_into()
                    .map_err(|_| serde::de::Error::custom(format!("Expected array of length {}, got {}", N, len)))
            })
            .transpose()
    }
}