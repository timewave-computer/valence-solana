// Session account state management for valence-kernel execution contexts
//
// Sessions represent isolated execution environments within the valence-kernel that
// maintain their own account borrow states, security policies, and operation history.
// This module defines the session account structure and implements the borrowing
// semantics that enable safe concurrent access to Solana accounts across operations.
//
// BORROWING SEMANTICS: Sessions track which accounts are currently borrowed and in
// what mode (read/write) to prevent conflicts and ensure atomic operations. The
// borrowing system enables session isolation while maintaining performance through
// efficient account access patterns.
//
// KERNEL INTEGRATION: Session accounts serve as the primary state container for
// kernel operations, storing namespace context, guard references, account lookup
// tables, and borrow state that enables the batch execution engine to operate
// safely across complex operation sequences.
//
// PERFORMANCE OPTIMIZATION: Fixed-size account arrays and efficient borrowing
// bitmaps ensure O(1) operations for account state management while preventing
// heap allocations during execution.
use crate::namespace::NamespacePath;
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
    
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.address == Pubkey::default()
    }
    
    #[must_use]
    pub const fn can_read(&self) -> bool {
        self.mode & crate::instructions::batch_operations::ACCESS_MODE_READ != 0
    }
    
    #[must_use]
    pub const fn can_write(&self) -> bool {
        self.mode & crate::instructions::batch_operations::ACCESS_MODE_WRITE != 0
    }
}

// ================================
// Core Session Structure
// ================================

/// Main session account that manages stateful operations with namespace-based authorization
#[account]
#[derive(Debug)]
pub struct Session {
    /// Namespace path for this session (e.g., "shard/session123")
    pub namespace: NamespacePath,
    
    /// Reference to guard account
    pub guard_account: Pubkey,
    
    /// Reference to account lookup table
    pub account_lookup: Pubkey,
    
    /// Owner of this session (has full control)
    pub owner: Pubkey,
    
    /// Shard that created this session
    pub shard: Pubkey,
    
    /// Optional parent session for hierarchical authorization
    pub parent_session: Option<Pubkey>,
    
    /// Usage counter for rate limiting
    pub usage_count: u64,
    
    /// Shard-specific metadata
    pub metadata: [u8; 64],
    
    /// Creation timestamp
    pub created_at: i64,
    
    /// Last update timestamp
    pub updated_at: i64,
    
    /// Currently borrowed accounts (up to 8 for minimal implementation)
    pub borrowed_accounts: [SessionBorrowedAccount; 8],
    
    /// Bitmap tracking which slots are occupied (bit n = slot n occupied)
    /// OPTIMIZATION: Bitmap allows O(1) lookups and atomic updates
    pub borrowed_bitmap: u8,
    
    /// Current CPI depth to prevent stack overflow
    /// 
    /// SECURITY: Limits recursive CPI calls to prevent stack exhaustion attacks
    /// Maximum depth of 4 aligns with Solana's practical limits while preventing
    /// malicious programs from creating infinite call chains
    pub cpi_depth: u8,
    
    /// Whether this session is active (can be invalidated for move semantics)
    pub active: bool,
    
    /// Nonce to invalidate cached references after ownership transfer
    pub nonce: u64,
    
    /// Child accounts created by this session (up to 8)
    pub child_accounts: [Pubkey; 8],
    
    /// Number of child accounts created
    pub child_count: u8,
}

impl Session {
    pub const LEN: usize = 8 +         // anchor discriminator
        256 + 2 +    // namespace path (fixed array + len)
        32 +         // guard_account
        32 +         // account_lookup
        32 +         // owner
        32 +         // shard
        1 + 32 +     // Option<parent_session>
        8 +          // usage_count
        64 +         // metadata
        8 +          // created_at
        8 +          // updated_at
        8 * 41 +     // borrowed_accounts array
        1 +          // borrowed_bitmap
        1 +          // cpi_depth
        1 +          // active
        8 +          // nonce
        8 * 32 +     // child_accounts array
        1;           // child_count

    /// Calculate space for account allocation
    #[must_use]
    pub const fn calculate_space() -> usize {
        Self::LEN
    }

    /// Find a free slot for borrowing an account
    #[must_use]
    pub const fn find_free_slot(&self) -> Option<usize> {
        let inverted = !self.borrowed_bitmap;
        if inverted == 0 {
            return None;
        }
        Some(inverted.trailing_zeros() as usize)
    }

    /// Check if an account is already borrowed
    #[must_use]
    pub fn is_borrowed(&self, account: &Pubkey) -> bool {
        self.borrowed_accounts
            .iter()
            .any(|b| !b.is_empty() && b.address == *account)
    }

    /// Get the index of a borrowed account
    #[must_use]
    pub fn get_borrowed_index(&self, account: &Pubkey) -> Option<usize> {
        self.borrowed_accounts
            .iter()
            .position(|b| !b.is_empty() && b.address == *account)
    }

    /// Borrow an account
    pub fn borrow_account(
        &mut self,
        account: Pubkey,
        mode: u8,
        clock: &Clock,
    ) -> Result<usize> {
        // Check if already borrowed
        if let Some(index) = self.get_borrowed_index(&account) {
            return Ok(index);
        }

        // Find free slot
        let slot = self
            .find_free_slot()
            .ok_or(crate::errors::KernelError::BorrowCapacityExceeded)?;

        // Borrow the account
        self.borrowed_accounts[slot] = SessionBorrowedAccount {
            address: account,
            borrowed_at: clock.unix_timestamp,
            mode,
        };

        // Update bitmap
        self.borrowed_bitmap |= 1 << slot;

        Ok(slot)
    }

    /// Release a borrowed account
    pub fn release_account(&mut self, account: &Pubkey) -> Result<()> {
        let index = self
            .get_borrowed_index(account)
            .ok_or(crate::errors::KernelError::AccountNotBorrowed)?;

        // Clear the slot
        self.borrowed_accounts[index] = SessionBorrowedAccount::EMPTY;

        // Update bitmap
        self.borrowed_bitmap &= !(1 << index);

        Ok(())
    }

    /// Release all borrowed accounts
    pub fn release_all_accounts(&mut self) {
        self.borrowed_accounts = [SessionBorrowedAccount::EMPTY; 8];
        self.borrowed_bitmap = 0;
    }

    /// Get the namespace for a child session
    pub fn child_namespace(&self, child_name: &str) -> Result<NamespacePath> {
        self.namespace.child(child_name)
    }

    /// Update session metadata
    pub fn set_metadata(&mut self, metadata: [u8; 64], clock: &Clock) {
        self.metadata = metadata;
        self.updated_at = clock.unix_timestamp;
    }

    /// Increment usage counter
    pub fn increment_usage(&mut self, clock: &Clock) -> Result<()> {
        self.usage_count = self.usage_count
            .checked_add(1)
            .ok_or(crate::errors::KernelError::UsageLimitExceeded)?;
        self.updated_at = clock.unix_timestamp;
        Ok(())
    }

    /// Create a new session
    pub fn new(
        params: CreateSessionParams,
        owner: Pubkey,
        shard: Pubkey,
        guard_account: Pubkey,
        account_lookup: Pubkey,
        clock: &Clock,
    ) -> Result<Self> {
        let path_str = std::str::from_utf8(&params.namespace_path[..params.namespace_path_len as usize])
            .map_err(|_| crate::errors::KernelError::NamespaceInvalidPath)?;
        let namespace = NamespacePath::new(path_str)?;
        
        Ok(Self {
            namespace,
            guard_account,
            account_lookup,
            owner,
            shard,
            parent_session: params.parent_session,
            usage_count: 0,
            metadata: params.metadata,
            created_at: clock.unix_timestamp,
            updated_at: clock.unix_timestamp,
            borrowed_accounts: [SessionBorrowedAccount::EMPTY; 8],
            borrowed_bitmap: 0,
            cpi_depth: 0,
            active: true,
            nonce: 0,
            child_accounts: [Pubkey::default(); 8],
            child_count: 0,
        })
    }
    
    /// Check and increment CPI depth
    pub fn check_and_increment_cpi_depth(&mut self) -> Result<()> {
        const MAX_CPI_DEPTH: u8 = 4;
        
        if self.cpi_depth >= MAX_CPI_DEPTH {
            return Err(crate::errors::KernelError::CrossProgramInvocationDepthExceeded.into());
        }
        
        self.cpi_depth = self.cpi_depth.saturating_add(1);
        Ok(())
    }
    
    /// Decrement CPI depth
    pub fn decrement_cpi_depth(&mut self) {
        self.cpi_depth = self.cpi_depth.saturating_sub(1);
    }
    
    /// Track a newly created child account
    pub fn track_child_account(&mut self, child: Pubkey) -> Result<()> {
        if self.child_count as usize >= 8 {
            return Err(crate::errors::KernelError::TooManyAccounts.into());
        }
        
        // Check if already tracked
        if self.is_child_account(&child) {
            return Ok(());
        }
        
        self.child_accounts[self.child_count as usize] = child;
        self.child_count += 1;
        Ok(())
    }
    
    /// Remove a child account from tracking
    /// 
    /// # Errors
    /// Returns error if the account is not found in child accounts
    pub fn untrack_child_account(&mut self, child: Pubkey) -> Result<()> {
        for i in 0..self.child_count as usize {
            if self.child_accounts[i] == child {
                // Move last element to this position and decrement count
                if i < (self.child_count as usize - 1) {
                    self.child_accounts[i] = self.child_accounts[self.child_count as usize - 1];
                }
                self.child_count -= 1;
                return Ok(());
            }
        }
        Err(crate::errors::KernelError::InvalidParameters.into())
    }
    
    /// Check if an account is a child of this session
    #[must_use]
    pub fn is_child_account(&self, account: &Pubkey) -> bool {
        self.child_accounts[..self.child_count as usize]
            .iter()
            .any(|child| child == account)
    }
}

// ================================
// Session Creation Parameters
// ================================

/// Parameters for creating a new session with namespace
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateSessionParams {
    /// Fixed-size namespace path
    pub namespace_path: [u8; 128],
    /// Length of the actual path
    pub namespace_path_len: u16,
    /// Initial metadata
    pub metadata: [u8; 64],
    /// Optional parent session
    pub parent_session: Option<Pubkey>,
}

// ================================
// Session Operations
// ================================

