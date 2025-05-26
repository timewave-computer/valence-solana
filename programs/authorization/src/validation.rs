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
        if timestamp < ValidationConstants::MIN_TIMESTAMP || timestamp > ValidationConstants::MAX_TIMESTAMP {
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

    /// Validate ZK-specific permissions for message execution
    pub fn validate_zk_permissions(
        zk_message_with_proof: &crate::zk_message::ZKMessageWithProof,
        sender: &Pubkey,
    ) -> Result<()> {
        let message = &zk_message_with_proof.message;
        let proof = &zk_message_with_proof.proof;
        
        // 1. Validate registry ID permissions
        // Only allow messages from registered ZK programs
        if message.registry_id == 0 {
            msg!("Invalid registry ID: cannot be zero");
            return Err(AuthorizationError::ZKProgramNotFound.into());
        }
        
        // 2. Validate cross-chain permissions
        // Ensure source and target chains are valid
        if message.source_chain == 0 || message.target_chain == 0 {
            msg!("Invalid chain IDs: source={}, target={}", message.source_chain, message.target_chain);
            return Err(AuthorizationError::InvalidParameters.into());
        }
        
        // 3. Validate payload size limits for ZK messages
        if message.payload.len() > 8192 { // 8KB limit for ZK message payloads
            msg!("ZK message payload too large: {} bytes", message.payload.len());
            return Err(AuthorizationError::PayloadTooLarge.into());
        }
        
        // 4. Validate proof size limits
        if proof.proof.len() > 16384 { // 16KB limit for ZK proofs
            msg!("ZK proof too large: {} bytes", proof.proof.len());
            return Err(AuthorizationError::InvalidZKProof.into());
        }
        
        // 5. Validate public inputs size
        if proof.public_inputs.len() > 1024 { // 1KB limit for public inputs
            msg!("ZK proof public inputs too large: {} bytes", proof.public_inputs.len());
            return Err(AuthorizationError::InvalidZKProof.into());
        }
        
        // 6. Validate nonce uniqueness (basic check)
        if message.nonce == 0 {
            msg!("Invalid nonce: cannot be zero");
            return Err(AuthorizationError::InvalidParameters.into());
        }
        
        // 7. Validate verification key ID format
        if proof.verification_key_id == Pubkey::default() {
            msg!("Invalid verification key ID: cannot be default pubkey");
            return Err(AuthorizationError::VerificationKeyNotFound.into());
        }
        
        // 8. Validate sender permissions for ZK messages
        // For ZK messages, we can be more permissive since the proof validates authenticity
        // But we still want to prevent spam from unauthorized senders
        if sender == &Pubkey::default() {
            msg!("Invalid sender: cannot be default pubkey");
            return Err(AuthorizationError::UnauthorizedSender.into());
        }
        
        // 9. Validate message structure integrity
        if !message.verify_hash() {
            msg!("ZK message hash verification failed");
            return Err(AuthorizationError::InvalidMessageFormat.into());
        }
        
        // 10. Additional ZK-specific validations
        // Check for known malicious patterns or blacklisted registry IDs
        let blacklisted_registries = [999999u64]; // Example blacklist
        if blacklisted_registries.contains(&message.registry_id) {
            msg!("Registry ID {} is blacklisted", message.registry_id);
            return Err(AuthorizationError::ZKProgramInactive.into());
        }
        
        msg!("ZK-specific permission validation passed for registry_id: {}, sender: {}", 
             message.registry_id, sender);
        
        Ok(())
    }
} 