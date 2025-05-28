// Comprehensive input validation utilities for authorization program
use anchor_lang::prelude::*;
use crate::error::AuthorizationError;

/// Validation constants
pub struct ValidationConstants;

impl ValidationConstants {
    /// Maximum label length
    pub const MAX_LABEL_LENGTH: usize = 32;
    /// Minimum label length
    pub const MIN_LABEL_LENGTH: usize = 1;
    /// Maximum allowed users in allowlist
    pub const MAX_ALLOWED_USERS: usize = 100;
    /// Maximum concurrent executions
    pub const MAX_CONCURRENT_EXECUTIONS: u32 = 1000;
    /// Minimum timestamp (Unix epoch)
    pub const MIN_TIMESTAMP: i64 = 0;
    /// Maximum timestamp (year 2100)
    pub const MAX_TIMESTAMP: i64 = 4102444800;
}

/// Validation utilities
pub struct Validator;

impl Validator {
    /// Validate authorization label
    pub fn validate_label(label: &str) -> Result<()> {
        if label.is_empty() {
            return Err(AuthorizationError::InvalidParameters.into());
        }
        
        if label.len() > ValidationConstants::MAX_LABEL_LENGTH {
            return Err(AuthorizationError::InvalidParameters.into());
        }
        
        // Check for valid characters (alphanumeric, underscore, hyphen)
        if !label.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(AuthorizationError::InvalidParameters.into());
        }
        
        Ok(())
    }
    
    /// Validate timestamp range
    pub fn validate_timestamp(timestamp: i64) -> Result<()> {
        if !(ValidationConstants::MIN_TIMESTAMP..=ValidationConstants::MAX_TIMESTAMP).contains(&timestamp) {
            return Err(AuthorizationError::InvalidParameters.into());
        }
        Ok(())
    }
    
    /// Validate timestamp ordering (not_before < expiration)
    pub fn validate_timestamp_ordering(not_before: i64, expiration: Option<i64>) -> Result<()> {
        Self::validate_timestamp(not_before)?;
        
        if let Some(exp) = expiration {
            Self::validate_timestamp(exp)?;
            if exp <= not_before {
                return Err(AuthorizationError::InvalidParameters.into());
            }
        }
        
        Ok(())
    }
    
    /// Validate allowed users list
    pub fn validate_allowed_users(allowed_users: &Option<Vec<Pubkey>>) -> Result<()> {
        if let Some(users) = allowed_users {
            if users.len() > ValidationConstants::MAX_ALLOWED_USERS {
                return Err(AuthorizationError::InvalidParameters.into());
            }
            
            // Check for duplicate users
            let mut unique_users = std::collections::HashSet::new();
            for user in users {
                if !unique_users.insert(user) {
                    return Err(AuthorizationError::InvalidParameters.into());
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate concurrent executions limit
    pub fn validate_concurrent_executions(max_concurrent: u32) -> Result<()> {
        if max_concurrent == 0 || max_concurrent > ValidationConstants::MAX_CONCURRENT_EXECUTIONS {
            return Err(AuthorizationError::InvalidParameters.into());
        }
        Ok(())
    }
    
    /// Validate authorization creation parameters
    pub fn validate_authorization_creation(
        label: &str,
        allowed_users: &Option<Vec<Pubkey>>,
        not_before: i64,
        expiration: Option<i64>,
        max_concurrent_executions: u32,
    ) -> Result<()> {
        Self::validate_label(label)?;
        Self::validate_allowed_users(allowed_users)?;
        Self::validate_timestamp_ordering(not_before, expiration)?;
        Self::validate_concurrent_executions(max_concurrent_executions)?;
        Ok(())
    }
    
    /// Validate message batch
    pub fn validate_message_batch(messages: &[crate::state::ProcessorMessage]) -> Result<()> {
        if messages.is_empty() {
            return Err(AuthorizationError::EmptyMessageBatch.into());
        }
        
        // Validate each message
        for message in messages {
            Self::validate_processor_message(message)?;
        }
        
        Ok(())
    }
    
    /// Validate individual processor message
    pub fn validate_processor_message(message: &crate::state::ProcessorMessage) -> Result<()> {
        // Validate program ID is not default
        if message.program_id == Pubkey::default() {
            return Err(AuthorizationError::InvalidParameters.into());
        }
        
        // Validate instruction data is not empty
        if message.data.is_empty() {
            return Err(AuthorizationError::InvalidParameters.into());
        }
        
        // Validate accounts list is not empty
        if message.accounts.is_empty() {
            return Err(AuthorizationError::InvalidParameters.into());
        }
        
        Ok(())
    }
    
    /// Validate ZK message hash
    pub fn validate_zk_message_hash(message_hash: &[u8; 32]) -> Result<()> {
        // Check that hash is not all zeros
        if message_hash.iter().all(|&b| b == 0) {
            return Err(AuthorizationError::InvalidParameters.into());
        }
        Ok(())
    }
    
    /// Validate sequence number ordering
    pub fn validate_sequence_ordering(current_sequence: u64, new_sequence: u64) -> Result<()> {
        if new_sequence <= current_sequence {
            return Err(AuthorizationError::InvalidParameters.into());
        }
        Ok(())
    }

    // ZK validation functions removed due to deleted zk_message module
    // These can be re-implemented when ZK message functionality is needed
} 