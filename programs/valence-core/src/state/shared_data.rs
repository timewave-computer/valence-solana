// Shared data structures for cross-session communication and state management
use crate::errors::ValenceError;
use anchor_lang::prelude::*;

/// Well-defined shared data structure for sessions
/// Provides common functionality across all session types
#[derive(Debug, Clone, Copy, AnchorSerialize, AnchorDeserialize)]
pub struct SessionSharedData {
    /// Version byte for future compatibility
    pub version: u8,
    
    /// Reentrancy flag (0 = not entered, 1 = entered)
    pub reentrancy_flag: u8,
    
    /// Current CPI depth
    pub cpi_depth: u8,
    
    /// Feature flags bitmap
    pub feature_flags: u8,
    
    /// Protocol-specific custom data storage
    /// Use this field for your protocol's session state
    pub custom_data: [u8; 32],
    
    /// Reserved space for future functionality
    pub _reserved: [u8; 220],
}

impl Default for SessionSharedData {
    fn default() -> Self {
        Self {
            version: 1,
            reentrancy_flag: 0,
            cpi_depth: 0,
            feature_flags: 0,
            custom_data: [0u8; 32],
            _reserved: [0u8; 220],
        }
    }
}

impl SessionSharedData {
    /// Current version number
    pub const CURRENT_VERSION: u8 = 1;
    
    /// Maximum allowed CPI depth
    pub const MAX_CPI_DEPTH: u8 = 4;
    
    /// Feature flag bit positions
    pub const FLAG_PAUSED: u8 = 1 << 0;
    pub const FLAG_DEBUG: u8 = 1 << 1;
    pub const FLAG_ATOMIC: u8 = 1 << 2;
    pub const FLAG_CROSS_PROTOCOL: u8 = 1 << 3;
    
    // ================================
    // Reentrancy Protection
    // ================================
    // Note: Transaction-wide reentrancy is prevented by Solana's runtime.
    // This flag protects against instruction-level reentrancy within a single
    // instruction's execution (e.g., through CPIs that call back).
    
    /// Check if currently in a reentrant call
    pub fn is_entered(&self) -> bool {
        self.reentrancy_flag != 0
    }
    
    /// Enter a protected section
    pub fn enter_protected_section(&mut self) -> Result<()> {
        require!(!self.is_entered(), ValenceError::InvalidStateTransition);
        self.reentrancy_flag = 1;
        Ok(())
    }
    
    /// Exit a protected section
    pub fn exit_protected_section(&mut self) {
        self.reentrancy_flag = 0;
    }
    
    // ================================
    // CPI Depth Management
    // ================================
    
    /// Get current CPI depth
    pub fn current_cpi_depth(&self) -> u8 {
        self.cpi_depth
    }
    
    /// Check if at maximum CPI depth
    pub fn is_at_max_cpi_depth(&self) -> bool {
        self.cpi_depth >= Self::MAX_CPI_DEPTH
    }
    
    /// Increment CPI depth with bounds checking
    pub fn check_and_increment_cpi_depth(&mut self) -> Result<()> {
        require!(self.cpi_depth < Self::MAX_CPI_DEPTH, ValenceError::CpiDepthExceeded);
        self.cpi_depth += 1;
        Ok(())
    }
    
    /// Decrement CPI depth
    pub fn decrement_cpi_depth(&mut self) {
        self.cpi_depth = self.cpi_depth.saturating_sub(1);
    }
    
    // ================================
    // Feature Flags
    // ================================
    
    /// Check if a feature flag is set
    pub fn has_flag(&self, flag: u8) -> bool {
        (self.feature_flags & flag) != 0
    }
    
    /// Set a feature flag
    pub fn set_flag(&mut self, flag: u8) {
        self.feature_flags |= flag;
    }
    
    /// Clear a feature flag
    pub fn clear_flag(&mut self, flag: u8) {
        self.feature_flags &= !flag;
    }
    
    /// Toggle a feature flag
    pub fn toggle_flag(&mut self, flag: u8) {
        self.feature_flags ^= flag;
    }
    
    /// Check if paused
    pub fn is_paused(&self) -> bool {
        self.has_flag(Self::FLAG_PAUSED)
    }
    
    /// Set paused state
    pub fn set_paused(&mut self, paused: bool) {
        if paused {
            self.set_flag(Self::FLAG_PAUSED);
        } else {
            self.clear_flag(Self::FLAG_PAUSED);
        }
    }
    
    // ================================
    // Custom Data Access
    // ================================
    
    /// Get custom data reference
    pub fn custom_data(&self) -> &[u8; 32] {
        &self.custom_data
    }
    
    /// Get mutable custom data reference
    pub fn custom_data_mut(&mut self) -> &mut [u8; 32] {
        &mut self.custom_data
    }
}

/// Ensure SessionSharedData is exactly 256 bytes for consistent account layout
const _: () = {
    assert!(
        std::mem::size_of::<SessionSharedData>() == 256,
        "SessionSharedData must be exactly 256 bytes"
    );
};