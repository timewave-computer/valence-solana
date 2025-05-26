// Validation utilities for Valence Solana programs
use anchor_lang::prelude::*;

/// Validation utilities
pub struct Validator;

impl Validator {
    /// Validate authorization label
    pub fn validate_label(label: &str) -> Result<()> {
        if label.is_empty() {
            return Err(ProgramError::InvalidArgument.into());
        }
        
        if label.len() > 32 {
            return Err(ProgramError::InvalidArgument.into());
        }
        
        // Check for valid characters (alphanumeric and underscore)
        if !label.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ProgramError::InvalidArgument.into());
        }
        
        Ok(())
    }
    
    /// Validate message batch
    pub fn validate_message_batch<T>(messages: &[T]) -> Result<()> {
        if messages.is_empty() {
            return Err(ProgramError::InvalidArgument.into());
        }
        
        if messages.len() > 100 {
            return Err(ProgramError::InvalidArgument.into());
        }
        
        Ok(())
    }
    
    /// Validate pubkey is not default
    pub fn validate_pubkey(pubkey: &Pubkey) -> Result<()> {
        if *pubkey == Pubkey::default() {
            return Err(ProgramError::InvalidArgument.into());
        }
        Ok(())
    }
    
    /// Validate amount is not zero
    pub fn validate_amount(amount: u64) -> Result<()> {
        if amount == 0 {
            return Err(ProgramError::InvalidArgument.into());
        }
        Ok(())
    }
} 