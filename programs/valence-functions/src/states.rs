// State definitions for shard-specific data structures in valence-functions
// Provides reusable state types for common shard patterns like escrow
use anchor_lang::prelude::*;

// ================================
// Core State Type System
// ================================

/// Trait for validating state consistency and business rules
/// All shard states should implement this for safety checks
pub trait StateValidator {
    /// Validate the state against business rules and constraints
    fn validate(&self) -> Result<()>;

    /// Check if the state allows the specified operation
    fn allows_operation(&self, operation: &str) -> bool;
}

// ================================
// Escrow Shard State
// ================================

/// Escrow state for atomic swaps and secure transactions
/// Manages the lifecycle of peer-to-peer trades with time-based expiration
#[account]
#[derive(Debug)]
pub struct EscrowState {
    /// Seller who initiated the escrow
    /// Cannot be changed once set - provides immutable ownership
    pub seller: Pubkey,

    /// Buyer who accepted the offer (None until accepted)
    /// Set when a buyer commits to the transaction
    pub buyer: Option<Pubkey>,

    /// Asset being sold (NFT mint or SPL token mint)
    /// Identifies the specific asset in the escrow
    pub asset_mint: Pubkey,

    /// Price in lamports or token amount
    /// Denominated based on the payment method configuration
    pub price: u64,

    /// Unix timestamp when escrow was created
    /// Used for tracking escrow age and analytics
    pub created_at: i64,

    /// Unix timestamp when escrow expires
    /// After this time, escrow can be cancelled by seller
    pub expires_at: i64,

    /// Escrow status for lifecycle management
    pub status: EscrowStatus,

    /// Reserved space for future upgrades
    /// Ensures forward compatibility without breaking account layout
    pub _reserved: [u8; 32],
}

/// Lifecycle status of an escrow transaction
/// Tracks the current state and allowed transitions
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum EscrowStatus {
    /// Escrow created, waiting for buyer
    Open,
    /// Buyer has committed, awaiting completion
    Committed,
    /// Transaction completed successfully
    Completed,
    /// Escrow cancelled (by seller or expiration)
    Cancelled,
}

impl Default for EscrowStatus {
    fn default() -> Self {
        Self::Open
    }
}

// ================================
// Escrow State Implementation
// ================================

impl EscrowState {
    /// Account space required for serialization
    pub const LEN: usize = 8 +      // anchor discriminator
        32 +     // seller pubkey
        1 + 32 + // Option<buyer> - 1 byte tag + 32 byte pubkey
        32 +     // asset_mint pubkey
        8 +      // price amount
        8 +      // created_at timestamp
        8 +      // expires_at timestamp
        1 +      // status enum (1 byte)
        32; // reserved space

    /// Maximum escrow duration (30 days in seconds)
    pub const MAX_DURATION: i64 = 30 * 24 * 60 * 60;

    /// Minimum escrow duration (1 hour in seconds)
    pub const MIN_DURATION: i64 = 60 * 60;

    /// Check if escrow has expired based on current time
    /// Returns true if current time exceeds expiration
    pub fn is_expired(&self, clock: &Clock) -> bool {
        clock.unix_timestamp >= self.expires_at
    }

    /// Check if escrow is ready for completion
    /// Requires a committed buyer and unexpired status
    pub fn is_ready_for_completion(&self, clock: &Clock) -> bool {
        self.buyer.is_some() && self.status == EscrowStatus::Committed && !self.is_expired(clock)
    }

    /// Check if escrow can be cancelled
    /// Allowed for seller if no buyer, or if expired
    pub fn can_be_cancelled(&self, caller: &Pubkey, clock: &Clock) -> bool {
        // Seller can always cancel their own escrow
        if *caller == self.seller {
            match self.status {
                EscrowStatus::Open => true,
                EscrowStatus::Committed => self.is_expired(clock),
                _ => false,
            }
        } else {
            false
        }
    }

    /// Get remaining time before expiration (in seconds)
    /// Returns 0 if already expired
    pub fn remaining_time(&self, clock: &Clock) -> i64 {
        (self.expires_at - clock.unix_timestamp).max(0)
    }

    /// Check if a specific user can interact with this escrow
    /// Based on role and current escrow status
    pub fn can_interact(&self, user: &Pubkey) -> bool {
        match self.status {
            EscrowStatus::Open => true, // Anyone can become buyer
            EscrowStatus::Committed => {
                // Only seller and committed buyer can interact
                *user == self.seller || self.buyer == Some(*user)
            }
            _ => false, // No interactions allowed for completed/cancelled
        }
    }

    /// Transition escrow to a new status
    /// Validates that the transition is allowed
    pub fn transition_status(&mut self, new_status: EscrowStatus) -> Result<()> {
        use EscrowStatus::*;

        let valid_transition = matches!(
            (&self.status, &new_status),
            (Open, Committed) | (Open, Cancelled) | (Committed, Completed) | (Committed, Cancelled)
        );

        require!(
            valid_transition,
            anchor_lang::error::ErrorCode::ConstraintRaw
        );

        self.status = new_status;
        Ok(())
    }
}

impl StateValidator for EscrowState {
    /// Validate escrow state consistency
    fn validate(&self) -> Result<()> {
        // Check price is positive
        require!(self.price > 0, anchor_lang::error::ErrorCode::ConstraintRaw);

        // Check timestamps are valid
        require!(
            self.created_at > 0 && self.expires_at > self.created_at,
            anchor_lang::error::ErrorCode::ConstraintRaw
        );

        // Check duration is within bounds
        let duration = self.expires_at - self.created_at;
        require!(
            (Self::MIN_DURATION..=Self::MAX_DURATION).contains(&duration),
            anchor_lang::error::ErrorCode::ConstraintRaw
        );

        // Check status consistency
        match self.status {
            EscrowStatus::Committed => {
                require!(
                    self.buyer.is_some(),
                    anchor_lang::error::ErrorCode::ConstraintRaw
                );
            }
            EscrowStatus::Open => {
                require!(
                    self.buyer.is_none(),
                    anchor_lang::error::ErrorCode::ConstraintRaw
                );
            }
            _ => {} // Other statuses don't have specific requirements
        }

        Ok(())
    }

    /// Check if the state allows the specified operation
    fn allows_operation(&self, operation: &str) -> bool {
        match operation {
            "commit" => self.status == EscrowStatus::Open,
            "complete" => self.status == EscrowStatus::Committed,
            "cancel" => matches!(self.status, EscrowStatus::Open | EscrowStatus::Committed),
            _ => false,
        }
    }
}

// ================================
// PDA Management System
// ================================

/// PDA seeds for deterministic address generation
/// Provides consistent seed values across the application
pub mod seeds {
    /// Seed for escrow state PDAs
    pub const ESCROW_STATE: &[u8] = b"escrow_state";
    /// Seed for escrow vault PDAs (token holding accounts)
    pub const ESCROW_VAULT: &[u8] = b"escrow_vault";
}

/// PDA derivation utilities for consistent address generation
/// Ensures deterministic account addresses across instructions
pub mod pda {
    use super::*;

    /// Derive escrow state PDA address
    /// Creates deterministic address based on seller, asset, and nonce
    pub fn escrow_state(
        seller: &Pubkey,
        asset_mint: &Pubkey,
        nonce: u64,
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                seeds::ESCROW_STATE,
                seller.as_ref(),
                asset_mint.as_ref(),
                &nonce.to_le_bytes(),
            ],
            program_id,
        )
    }

    /// Derive escrow vault PDA address for holding tokens
    /// Creates deterministic address for token custody
    pub fn escrow_vault(escrow_state: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[seeds::ESCROW_VAULT, escrow_state.as_ref()], program_id)
    }
}

// ================================
// State Creation Utilities
// ================================

/// Builder pattern for creating new escrow states
/// Provides a fluent interface for escrow configuration
pub struct EscrowBuilder {
    seller: Option<Pubkey>,
    asset_mint: Option<Pubkey>,
    price: Option<u64>,
    duration: Option<i64>,
}

impl EscrowBuilder {
    /// Create a new escrow builder
    #[must_use]
    pub const fn new() -> Self {
        Self {
            seller: None,
            asset_mint: None,
            price: None,
            duration: None,
        }
    }

    /// Set the seller for the escrow
    #[must_use]
    pub fn seller(mut self, seller: Pubkey) -> Self {
        self.seller = Some(seller);
        self
    }

    /// Set the asset mint for the escrow
    #[must_use]
    pub fn asset_mint(mut self, asset_mint: Pubkey) -> Self {
        self.asset_mint = Some(asset_mint);
        self
    }

    /// Set the price for the escrow
    #[must_use]
    pub fn price(mut self, price: u64) -> Self {
        self.price = Some(price);
        self
    }

    /// Set the duration for the escrow (in seconds)
    #[must_use]
    pub fn duration(mut self, duration: i64) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Build the escrow state with current timestamp
    pub fn build(self, clock: &Clock) -> Result<EscrowState> {
        let seller = self
            .seller
            .ok_or(anchor_lang::error::ErrorCode::ConstraintRaw)?;
        let asset_mint = self
            .asset_mint
            .ok_or(anchor_lang::error::ErrorCode::ConstraintRaw)?;
        let price = self
            .price
            .ok_or(anchor_lang::error::ErrorCode::ConstraintRaw)?;
        let duration = self.duration.unwrap_or(24 * 60 * 60); // Default 24 hours

        let escrow = EscrowState {
            seller,
            buyer: None,
            asset_mint,
            price,
            created_at: clock.unix_timestamp,
            expires_at: clock.unix_timestamp + duration,
            status: EscrowStatus::default(),
            _reserved: [0u8; 32],
        };

        // Validate the built escrow
        escrow.validate()?;

        Ok(escrow)
    }
}

impl Default for EscrowBuilder {
    fn default() -> Self {
        Self::new()
    }
}
