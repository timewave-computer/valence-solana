// Validation utilities for input sanitization and constraint checking
use anchor_lang::prelude::*;

/// Input validation utilities
pub struct Validator;

impl Validator {
    /// Validate authorization label format
    pub fn validate_label(label: &str) -> Result<()> {
        if label.is_empty() || label.len() > 64 {
            return Err(ProgramError::InvalidArgument.into());
        }
        
        // Only allow alphanumeric and basic punctuation
        if !label.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(ProgramError::InvalidArgument.into());
        }
        
        Ok(())
    }
    
    /// Validate message batch constraints
    pub fn validate_message_batch<T>(messages: &[T]) -> Result<()> {
        if messages.is_empty() {
            return Err(ProgramError::InvalidArgument.into());
        }
        
        if messages.len() > 32 {
            return Err(ProgramError::InvalidArgument.into());
        }
        
        Ok(())
    }
    
    /// Validate pubkey is not default/zero
    pub fn validate_pubkey(pubkey: &Pubkey) -> Result<()> {
        if *pubkey == Pubkey::default() {
            return Err(ProgramError::InvalidArgument.into());
        }
        Ok(())
    }
    
    /// Validate timestamp is reasonable (not too far in past/future)
    pub fn validate_timestamp(timestamp: i64) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let max_drift = 3600; // 1 hour
        
        if timestamp < now - max_drift || timestamp > now + max_drift {
            return Err(ProgramError::InvalidArgument.into());
        }
        
        Ok(())
    }
} 