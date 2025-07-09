/// Utilities and Helpers for Valence SDK
/// 
/// This module provides supporting utilities including address derivation,
/// testing helpers, development tools, and general utility functions.

pub mod addresses;
pub mod dev_tools;
pub mod testing;

pub use addresses::*;
pub use dev_tools::*;
pub use testing::*;

use crate::{ValenceResult, ValenceError};
use solana_sdk::pubkey::Pubkey;

/// Validate capability ID format
pub fn validate_capability_id(id: &str) -> ValenceResult<()> {
    if id.is_empty() {
        return Err(ValenceError::InvalidInputParameters("Capability ID cannot be empty".to_string()));
    }
    
    if !id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(ValenceError::InvalidInputParameters("Capability ID can only contain alphanumeric characters, underscores, and hyphens".to_string()));
    }
    
    Ok(())
}

/// Validate library ID format
pub fn validate_library_id(id: &str) -> ValenceResult<()> {
    if id.is_empty() {
        return Err(ValenceError::InvalidInputParameters("Library ID cannot be empty".to_string()));
    }
    
    if !id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.') {
        return Err(ValenceError::InvalidInputParameters("Library ID can only contain alphanumeric characters, underscores, hyphens, and dots".to_string()));
    }
    
    Ok(())
}

/// Validate namespace format
pub fn validate_namespace(namespace: &str) -> ValenceResult<()> {
    if namespace.is_empty() {
        return Err(ValenceError::InvalidInputParameters("Namespace cannot be empty".to_string()));
    }
    
    if !namespace.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(ValenceError::InvalidInputParameters("Namespace can only contain alphanumeric characters, underscores, and hyphens".to_string()));
    }
    
    Ok(())
}

/// Derive a PDA for a given seed and program ID
pub fn derive_pda(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(seeds, program_id)
}

/// Convert string to Pubkey
pub fn string_to_pubkey(s: &str) -> ValenceResult<Pubkey> {
    s.parse::<Pubkey>()
        .map_err(|_| ValenceError::InvalidInputParameters(format!("Invalid pubkey: {}", s)))
}

/// Get current timestamp
pub fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Convert timestamp to string
pub fn timestamp_to_string(timestamp: i64) -> String {
    use chrono::{DateTime, Utc};
    let dt = DateTime::<Utc>::from_timestamp(timestamp, 0).unwrap_or_default();
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Calculate SHA256 hash
pub fn sha256(data: &[u8]) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Calculate metadata hash for library
pub fn calculate_metadata_hash(name: &str, version: &str, description: &str, tags: &[String]) -> [u8; 32] {
    let mut data = String::new();
    data.push_str(name);
    data.push_str(version);
    data.push_str(description);
    for tag in tags {
        data.push_str(tag);
    }
    sha256(data.as_bytes())
}