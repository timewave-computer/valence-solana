// Move Semantics Patterns for Valence
//
// This module provides patterns and utilities for implementing move semantics
// in your protocols. Since Valence doesn't enforce a specific ownership model,
// these are optional patterns you can use to achieve clean ownership transfer.

use anchor_lang::prelude::*;
use crate::{SdkError, Result};
use solana_sdk::instruction::Instruction;

/// Trait for implementing move semantics on your protocol's accounts
pub trait MoveSemantics {
    /// Transfer ownership to a new owner, invalidating previous access
    fn transfer_ownership(&mut self, new_owner: Pubkey) -> Result<()>;
    
    /// Verify that the given owner has access
    fn verify_ownership(&self, owner: &Pubkey) -> Result<()>;
    
    /// Get the current nonce (for invalidating cached references)
    fn nonce(&self) -> u64;
}

/// Helper macro to implement basic move semantics for an account type
#[macro_export]
macro_rules! implement_move_semantics {
    ($account_type:ty) => {
        impl $crate::MoveSemantics for $account_type {
            fn transfer_ownership(&mut self, new_owner: Pubkey) -> Result<()> {
                self.owner = new_owner;
                self.nonce = self.nonce.saturating_add(1);
                Ok(())
            }
            
            fn verify_ownership(&self, owner: &Pubkey) -> Result<()> {
                if self.owner != *owner {
                    return Err($crate::SdkError::Unauthorized);
                }
                Ok(())
            }
            
            fn nonce(&self) -> u64 {
                self.nonce
            }
        }
    };
}

/// Pattern 1: Invalidation-based move
/// 
/// Use this when you want to transfer ownership by invalidating the old owner's access
pub struct InvalidationMove;

impl InvalidationMove {
    /// Transfer ownership by invalidating the old session
    pub fn transfer_with_invalidation(
        active: &mut bool,
        nonce: &mut u64,
        _new_owner: Pubkey,
    ) -> Result<()> {
        // Verify old session is active
        if !*active {
            return Err(SdkError::SessionInactive);
        }
        
        // Invalidate old session
        *active = false;
        *nonce = nonce.saturating_add(1);
        
        // New owner must create a new session
        Ok(())
    }
}

/// Pattern 2: Close-and-recreate move
/// 
/// Use this when you want to completely close the old account and create a new one
pub struct CloseAndRecreateMove;

impl CloseAndRecreateMove {
    /// Helper to extract rent from a closed account
    pub fn close_account<'info>(
        account: AccountInfo<'info>,
        recipient: AccountInfo<'info>,
    ) -> Result<()> {
        let dest_starting_lamports = recipient.lamports();
        **recipient.lamports.borrow_mut() = dest_starting_lamports
            .checked_add(account.lamports())
            .ok_or(SdkError::Overflow)?;
        **account.lamports.borrow_mut() = 0;
        
        account.assign(&System::id());
        account.resize(0)?;
        
        Ok(())
    }
}

/// Pattern 3: Versioned ownership
/// 
/// Use this when you want to track ownership changes over time
#[derive(Debug, Clone, Copy, AnchorSerialize, AnchorDeserialize)]
pub struct OwnershipVersion {
    pub owner: Pubkey,
    pub version: u64,
    pub transferred_at: i64,
}

impl OwnershipVersion {
    pub fn new(owner: Pubkey, clock: &Clock) -> Self {
        Self {
            owner,
            version: 1,
            transferred_at: clock.unix_timestamp,
        }
    }
    
    pub fn transfer(&mut self, new_owner: Pubkey, clock: &Clock) {
        self.owner = new_owner;
        self.version = self.version.saturating_add(1);
        self.transferred_at = clock.unix_timestamp;
    }
    
    pub fn verify(&self, owner: &Pubkey, min_version: Option<u64>) -> Result<()> {
        if self.owner != *owner {
            return Err(SdkError::Unauthorized);
        }
        
        if let Some(min) = min_version {
            if self.version < min {
                return Err(SdkError::StaleReference);
            }
        }
        
        Ok(())
    }
}

/// Pattern 4: Capability-based access (without committing to specific capability types)
/// 
/// Your protocol defines what capabilities mean
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct AccessCapability {
    pub granted_to: Pubkey,
    pub granted_by: Pubkey,
    pub data: [u8; 32], // Protocol-specific capability data
    pub expires_at: Option<i64>,
    pub uses_remaining: Option<u32>,
    pub nonce: u64,
}

impl AccessCapability {
    pub fn new(granted_to: Pubkey, granted_by: Pubkey, data: [u8; 32]) -> Self {
        Self {
            granted_to,
            granted_by,
            data,
            expires_at: None,
            uses_remaining: None,
            nonce: 0,
        }
    }
    
    pub fn with_expiry(mut self, expires_at: i64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    pub fn with_uses(mut self, uses: u32) -> Self {
        self.uses_remaining = Some(uses);
        self
    }
    
    pub fn verify(&self, user: &Pubkey, clock: &Clock) -> Result<()> {
        if self.granted_to != *user {
            return Err(SdkError::Unauthorized);
        }
        
        if let Some(expires_at) = self.expires_at {
            if clock.unix_timestamp >= expires_at {
                return Err(SdkError::CapabilityExpired);
            }
        }
        
        if let Some(uses) = self.uses_remaining {
            if uses == 0 {
                return Err(SdkError::CapabilityExhausted);
            }
        }
        
        Ok(())
    }
    
    pub fn use_once(&mut self) -> Result<()> {
        if let Some(uses) = &mut self.uses_remaining {
            *uses = uses.checked_sub(1).ok_or(SdkError::CapabilityExhausted)?;
        }
        Ok(())
    }
    
    pub fn revoke(&mut self) {
        self.nonce = self.nonce.saturating_add(1);
        self.uses_remaining = Some(0);
    }
}

/// Helper for creating atomic ownership transfers in transactions
pub struct AtomicOwnershipTransfer;

impl AtomicOwnershipTransfer {
    /// Build instructions for atomic ownership transfer
    /// Your protocol implements the actual transfer logic
    pub fn build_transfer_instructions(
        _old_owner: Pubkey,
        _new_owner: Pubkey,
        _resource: Pubkey,
    ) -> Vec<Instruction> {
        // Protocol-specific implementation
        vec![]
    }
}