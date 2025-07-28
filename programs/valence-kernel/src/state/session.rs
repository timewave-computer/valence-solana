// Session management for stateful operations and guard integration
use super::shared_data::SessionSharedData;
use anchor_lang::prelude::*;

// ================================
// Borrowed Account Tracking
// ================================

/// Tracks an account borrowed by a session
#[derive(Debug, Clone, Copy, AnchorSerialize, AnchorDeserialize, Default)]
pub struct SessionBorrowedAccount {
    /// The borrowed account's address
    pub address: Pubkey,
    /// When this account was borrowed
    pub borrowed_at: i64,
    /// Borrow mode flags (bit 0: read, bit 1: write)
    pub mode: u8,
}

impl SessionBorrowedAccount {
    pub const EMPTY: Self = Self {
        address: Pubkey::new_from_array([0u8; 32]),
        borrowed_at: 0,
        mode: 0,
    };
    
    pub fn is_empty(&self) -> bool {
        self.address == Pubkey::default()
    }
    
    pub fn can_read(&self) -> bool {
        self.mode & 1 != 0
    }
    
    pub fn can_write(&self) -> bool {
        self.mode & 2 != 0
    }
}

// ================================
// Session Scope Definitions
// ================================

/// Session scope determines the authorization context and hierarchy
#[derive(Debug, Clone, Copy, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum SessionContextScope {
    /// User-specific operations with individual permissions
    User,
    /// Shard-wide operations with global authority
    Global,
    /// Pool-specific operations scoped to a liquidity pool
    Pool(Pubkey),
    /// Token-specific operations scoped to a token mint
    Token(Pubkey),
    /// DAO governance operations scoped to a DAO
    Dao(Pubkey),
    /// Custom scope for shard-specific contexts
    Custom {
        /// Your shard's program ID
        shard: Pubkey,
        /// Your custom scope identifier (e.g., vault_id, market_id, etc.)
        scope_id: [u8; 32],
    },
}

// ================================
// Core Session Structure
// ================================

/// Main session account that manages stateful operations
/// Sessions provide controlled access to multiple program states through guards
#[account]
#[derive(Debug)]
pub struct Session {
    /// Session scope determines authorization context
    pub scope: SessionContextScope,
    /// Reference to guard data account (replaces inline guard)
    pub guard_data: Pubkey,
    /// Owner of this session (has full control)
    pub owner: Pubkey,
    /// Shard that created this session (formerly protocol)
    pub shard: Pubkey,
    /// Optional binding to another session for hierarchical authorization
    pub bound_to: Option<Pubkey>,
    /// Usage counter for rate limiting
    pub usage_count: u64,
    /// Shared data for cross-session communication
    pub shared_data: SessionSharedData,
    /// Shard-specific metadata
    pub metadata: [u8; 64],
    /// Creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
    /// Currently borrowed accounts (up to 8 for minimal implementation)
    pub borrowed_accounts: [SessionBorrowedAccount; 8],
    /// Bitmap tracking which slots are occupied (bit n = slot n occupied)
    pub borrowed_bitmap: u8,
}

// ================================
// Session Creation Parameters
// ================================

/// Complete parameters required for session creation
/// Encapsulates all configuration needed to initialize a new session
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateSessionParams {
    /// Session scope
    pub scope: SessionContextScope,
    /// Reference to guard data account
    pub guard_data: Pubkey,
    /// Optional binding to another session for hierarchical authorization
    pub bound_to: Option<Pubkey>,
    /// Initial shared data configuration
    pub shared_data: SessionSharedData,
    /// Shard-specific metadata
    pub metadata: [u8; 64],
}

// ================================
// Session Implementation
// ================================

impl Session {
    /// Length constant for Anchor account space allocation
    pub const LEN: usize = Self::calculate_space();
    
    /// Create a new session
    #[must_use]
    pub fn new(
        params: CreateSessionParams,
        owner: Pubkey,
        shard: Pubkey,
        clock: &Clock,
    ) -> Self {
        Self {
            scope: params.scope,
            guard_data: params.guard_data,
            owner,
            shard,
            bound_to: params.bound_to,
            usage_count: 0,
            shared_data: params.shared_data,
            metadata: params.metadata,
            created_at: clock.unix_timestamp,
            updated_at: clock.unix_timestamp,
            borrowed_accounts: [SessionBorrowedAccount::EMPTY; 8],
            borrowed_bitmap: 0,
        }
    }

    /// Check if this is a global scope session
    pub fn is_global(&self) -> bool {
        matches!(self.scope, SessionContextScope::Global)
    }

    /// Get the parent session reference if any
    pub fn bound_to(&self) -> Option<&Pubkey> {
        self.bound_to.as_ref()
    }


    /// Increment usage count
    pub fn increment_usage(&mut self, clock: &Clock) -> Result<()> {
        self.usage_count = self
            .usage_count
            .checked_add(1)
            .ok_or(crate::errors::KernelError::UsageLimitExceeded)?;
        self.updated_at = clock.unix_timestamp;
        Ok(())
    }

    /// Update metadata
    pub fn set_metadata(&mut self, metadata: [u8; 64], clock: &Clock) {
        self.metadata = metadata;
        self.updated_at = clock.unix_timestamp;
    }
    
    /// Borrow an account for this session
    pub fn borrow_account(&mut self, account: Pubkey, mode: u8, clock: &Clock) -> Result<()> {
        // Check if already borrowed by scanning only occupied slots (optimized)
        let mut bitmap = self.borrowed_bitmap;
        let mut i = 0;
        while bitmap != 0 {
            if bitmap & 1 != 0 && self.borrowed_accounts[i].address == account {
                return Err(crate::errors::KernelError::AccountAlreadyBorrowed.into());
            }
            bitmap >>= 1;
            i += 1;
        }
        
        // Find first free slot using bit manipulation (O(1))
        // Use bit trick: ~bitmap gives us inverted bits (free slots)
        let free_slots = !self.borrowed_bitmap;
        if free_slots == 0 {
            return Err(crate::errors::KernelError::BorrowCapacityExceeded.into());
        }
        
        // trailing_zeros gives us the position of the first free slot
        let slot = free_slots.trailing_zeros() as usize;
        
        // Safety check (should never happen with u8 bitmap)
        if slot >= 8 {
            return Err(crate::errors::KernelError::BorrowCapacityExceeded.into());
        }
        
        // Add to borrowed list
        self.borrowed_accounts[slot] = SessionBorrowedAccount {
            address: account,
            borrowed_at: clock.unix_timestamp,
            mode,
        };
        // Set bit in bitmap
        self.borrowed_bitmap |= 1 << slot;
        self.updated_at = clock.unix_timestamp;
        
        Ok(())
    }
    
    /// Check if an account is borrowed by this session (optimized)
    pub fn has_borrowed(&self, account: &Pubkey) -> Option<&SessionBorrowedAccount> {
        // Fast path: no accounts borrowed
        if self.borrowed_bitmap == 0 {
            return None;
        }
        
        // Check only set bits using bit manipulation
        let mut bitmap = self.borrowed_bitmap;
        let mut i = 0;
        while bitmap != 0 {
            if bitmap & 1 != 0 && self.borrowed_accounts[i].address == *account {
                return Some(&self.borrowed_accounts[i]);
            }
            bitmap >>= 1;
            i += 1;
        }
        None
    }
    
    /// Release a borrowed account (optimized)
    pub fn release_account(&mut self, account: &Pubkey, clock: &Clock) -> Result<()> {
        // Fast path: no accounts borrowed
        if self.borrowed_bitmap == 0 {
            return Err(crate::errors::KernelError::AccountNotBorrowed.into());
        }
        
        // Find the account using optimized bit scanning
        let mut bitmap = self.borrowed_bitmap;
        let mut i = 0;
        let mut slot = None;
        while bitmap != 0 {
            if bitmap & 1 != 0 && self.borrowed_accounts[i].address == *account {
                slot = Some(i);
                break;
            }
            bitmap >>= 1;
            i += 1;
        }
        
        let i = slot.ok_or(crate::errors::KernelError::AccountNotBorrowed)?;
        
        // Clear the slot - O(1) operation
        self.borrowed_accounts[i] = SessionBorrowedAccount::EMPTY;
        // Clear bit in bitmap
        self.borrowed_bitmap &= !(1 << i);
        self.updated_at = clock.unix_timestamp;
        
        Ok(())
    }
    
    /// Release all borrowed accounts
    pub fn release_all(&mut self, clock: &Clock) {
        // Clear all slots only if occupied
        for i in 0..8 {
            if (self.borrowed_bitmap & (1 << i)) != 0 {
                self.borrowed_accounts[i] = SessionBorrowedAccount::EMPTY;
            }
        }
        // Clear entire bitmap
        self.borrowed_bitmap = 0;
        self.updated_at = clock.unix_timestamp;
    }

    /// Calculate the space required for this session account
    pub const fn calculate_space() -> usize {
        8 + // discriminator
        64 + // scope (max size for enum with Custom variant)
        32 + // guard_data (Pubkey reference)
        32 + // owner
        32 + // shard  
        33 + // Option<bound_to> (1 + 32)
        8 + // usage_count
        256 + // shared_data
        64 + // metadata
        8 + // created_at
        8 + // updated_at
        8 * (32 + 8 + 1) + // borrowed_accounts array (8 * SessionBorrowedAccount size)
        1 // borrowed_bitmap
    }
}
