// Simplified escrow protocol for atomic swaps and secure trading
use crate::{Protocol, states::EscrowStatus};
use anchor_lang::prelude::*;

// ================================
// Core Escrow Protocol
// ================================

/// Simple escrow protocol configuration
#[derive(Clone, Debug)]
pub struct EscrowProtocol {
    /// Service fee in basis points (1 bp = 0.01%)
    pub fee_bps: u16,
    /// Default expiry duration in seconds
    pub default_expiry_seconds: i64,
    /// Maximum allowed expiry duration
    pub max_expiry_seconds: i64,
    /// Minimum trade value to prevent dust
    pub minimum_trade_value: u64,
}

impl Default for EscrowProtocol {
    fn default() -> Self {
        Self {
            fee_bps: 100,                       // 1% fee
            default_expiry_seconds: 24 * 3600,  // 24 hours
            max_expiry_seconds: 30 * 24 * 3600, // 30 days
            minimum_trade_value: 1_000,         // Minimum value
        }
    }
}

impl Protocol for EscrowProtocol {
    fn id(&self) -> [u8; 32] {
        let mut id = [0u8; 32];
        id[..6].copy_from_slice(b"ESCROW");
        id[6] = (self.fee_bps / 256) as u8;
        id[7] = (self.fee_bps % 256) as u8;
        id
    }

    fn version(&self) -> u16 {
        1
    }

    fn name(&self) -> &'static str {
        "Valence Escrow Protocol V1"
    }
}

// ================================
// Protocol Factory Functions
// ================================

/// Create a peer-to-peer escrow protocol instance
/// This is a convenience function for creating standard P2P escrow configurations
pub fn p2p_escrow() -> EscrowProtocol {
    EscrowProtocol::default()
}

// ================================
// Protocol Operations
// ================================

impl EscrowProtocol {
    /// Calculate escrow fees
    pub fn calculate_fees(&self, amount: u64) -> Result<u64> {
        (amount as u128)
            .checked_mul(self.fee_bps as u128)
            .and_then(|x| x.checked_div(10_000))
            .map(|x| x as u64)
            .ok_or_else(|| error!(EscrowError::MathOverflow))
    }

    /// Validate expiry duration
    pub fn validate_expiry(&self, duration_seconds: i64) -> Result<()> {
        require!(duration_seconds > 0, EscrowError::InvalidExpiry);
        require!(
            duration_seconds <= self.max_expiry_seconds,
            EscrowError::ExpiryTooLong
        );
        Ok(())
    }

    /// Validate trade amount
    pub fn validate_trade_amount(&self, amount: u64) -> Result<()> {
        require!(
            amount >= self.minimum_trade_value,
            EscrowError::TradeTooSmall
        );
        Ok(())
    }

    /// Calculate expiry timestamp
    pub fn calculate_expiry(&self, created_at: i64, duration: Option<i64>) -> Result<i64> {
        let duration_seconds = duration.unwrap_or(self.default_expiry_seconds);
        self.validate_expiry(duration_seconds)?;
        created_at
            .checked_add(duration_seconds)
            .ok_or_else(|| error!(EscrowError::MathOverflow))
    }
}

// ================================
// Additional Status Methods 
// ================================

impl EscrowStatus {
    /// Check if state transition is valid
    pub fn can_transition_to(&self, new_status: &EscrowStatus) -> bool {
        use EscrowStatus::*;
        matches!(
            (self, new_status),
            (Open, Committed) | (Open, Cancelled) | (Committed, Completed) | (Committed, Cancelled)
        )
    }

    /// Check if escrow is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, EscrowStatus::Completed | EscrowStatus::Cancelled)
    }
}

// ================================
// Protocol Presets
// ================================

impl EscrowProtocol {
    /// Create protocol for NFT trading
    pub fn nft_trading() -> Self {
        Self {
            fee_bps: 250,                       // 2.5% fee
            default_expiry_seconds: 7 * 24 * 3600,  // 7 days
            max_expiry_seconds: 30 * 24 * 3600, // 30 days
            minimum_trade_value: 100_000,       // Higher minimum
        }
    }

    /// Create protocol for token trading
    pub fn token_trading() -> Self {
        Self {
            fee_bps: 30,                        // 0.3% fee
            default_expiry_seconds: 3600,       // 1 hour
            max_expiry_seconds: 7 * 24 * 3600,  // 7 days
            minimum_trade_value: 1_000,         // Low minimum
        }
    }

    /// Create protocol for P2P trading
    pub fn p2p_trading() -> Self {
        Self {
            fee_bps: 0,                         // No fees
            default_expiry_seconds: 3600,       // 1 hour
            max_expiry_seconds: 24 * 3600,      // 24 hours
            minimum_trade_value: 100,           // Very low minimum
        }
    }
}

// ================================
// Errors
// ================================

#[error_code]
pub enum EscrowError {
    #[msg("Math overflow")]
    MathOverflow,

    #[msg("Invalid expiry duration")]
    InvalidExpiry,

    #[msg("Expiry duration too long")]
    ExpiryTooLong,

    #[msg("Trade amount too small")]
    TradeTooSmall,

    #[msg("Invalid state transition")]
    InvalidStateTransition,
}