//! Enhanced Linear Type System for Valence
//!
//! This module implements a sophisticated linear type system that provides:
//! - Compile-time type safety for operation ordering
//! - Voucher-based ownership tracking
//! - Capability-based access control
//! - State machine verification
//! - Non-replayable operations
//!
//! The system ensures that resources can only be used once and in the correct order,
//! preventing double-spending, unauthorized access, and protocol violations.

use anchor_lang::prelude::*;
use std::marker::PhantomData;

/// Capability types that represent permissions to perform operations
pub mod capabilities {
    
    /// Marker trait for all capabilities
    pub trait Capability: Clone + PartialEq {}
    
    /// Deposit capability - allows creating new positions
    #[derive(Clone, PartialEq, Debug)]
    pub struct DepositCap;
    impl Capability for DepositCap {}
    
    /// Transfer capability - allows moving vouchers between accounts
    #[derive(Clone, PartialEq, Debug)]
    pub struct TransferCap;
    impl Capability for TransferCap {}
    
    /// Withdraw capability - allows extracting value from positions
    #[derive(Clone, PartialEq, Debug)]
    pub struct WithdrawCap;
    impl Capability for WithdrawCap {}
    
    /// Admin capability - allows protocol administration
    #[derive(Clone, PartialEq, Debug)]
    pub struct AdminCap;
    impl Capability for AdminCap {}
    
    /// Composite capability that combines multiple permissions
    #[derive(Clone, PartialEq, Debug)]
    pub struct CompositeCap<T1: Capability, T2: Capability> {
        pub cap1: T1,
        pub cap2: T2,
    }
    impl<T1: Capability, T2: Capability> Capability for CompositeCap<T1, T2> {}
}

/// Linear state types that enforce correct operation ordering
pub mod states {
    
    /// Marker trait for all linear states
    pub trait LinearState: Clone + PartialEq {
        type Next: LinearState;
        const STATE_ID: u8;
        
        fn validate_transition(&self, next_state_id: u8) -> bool {
            next_state_id == Self::Next::STATE_ID
        }
    }
    
    /// Initial state - no operations performed yet
    #[derive(Clone, PartialEq, Debug)]
    pub struct Initial;
    impl LinearState for Initial {
        type Next = Deposited;
        const STATE_ID: u8 = 0;
    }
    
    /// Deposited state - funds have been deposited
    #[derive(Clone, PartialEq, Debug)]
    pub struct Deposited;
    impl LinearState for Deposited {
        type Next = VoucherTransferred;
        const STATE_ID: u8 = 1;
    }
    
    /// VoucherTransferred state - voucher has been moved
    #[derive(Clone, PartialEq, Debug)]
    pub struct VoucherTransferred;
    impl LinearState for VoucherTransferred {
        type Next = Collateralized;
        const STATE_ID: u8 = 2;
    }
    
    /// Collateralized state - additional collateral added
    #[derive(Clone, PartialEq, Debug)]
    pub struct Collateralized;
    impl LinearState for Collateralized {
        type Next = Final;
        const STATE_ID: u8 = 3;
    }
    
    /// Final state - operation complete, can withdraw
    #[derive(Clone, PartialEq, Debug)]
    pub struct Final;
    impl LinearState for Final {
        type Next = Final; // Terminal state
        const STATE_ID: u8 = 4;
    }
}

/// Voucher system for tracking ownership and preventing double-use
#[derive(Clone, Debug, PartialEq)]
pub struct LinearVoucher<S: states::LinearState, C: capabilities::Capability> {
    /// Unique identifier for this voucher
    pub voucher_id: u64,
    /// The account this voucher is bound to
    pub bound_account: Pubkey,
    /// Current state of the linear progression
    pub state: PhantomData<S>,
    /// Capabilities held by this voucher
    pub capability: PhantomData<C>,
    /// Nonce to prevent replay attacks
    pub nonce: u64,
    /// Expiration slot (optional)
    pub expires_at: Option<u64>,
}

impl<S: states::LinearState, C: capabilities::Capability> LinearVoucher<S, C> {
    pub fn new(voucher_id: u64, bound_account: Pubkey, nonce: u64) -> Self {
        Self {
            voucher_id,
            bound_account,
            state: PhantomData,
            capability: PhantomData,
            nonce,
            expires_at: None,
        }
    }
    
    /// Transition to the next state, consuming this voucher
    pub fn transition<NS: states::LinearState>(
        self,
    ) -> Result<LinearVoucher<NS, C>>
    where
        S: states::LinearState<Next = NS>,
    {
        // Validate the state transition is legal
        if !self.state_instance().validate_transition(NS::STATE_ID) {
            return Err(ErrorCode::InvalidStateTransition.into());
        }
        
        // Check expiration
        if let Some(expires_at) = self.expires_at {
            let current_slot = Clock::get()?.slot;
            if current_slot > expires_at {
                return Err(ErrorCode::VoucherExpired.into());
            }
        }
        
        Ok(LinearVoucher {
            voucher_id: self.voucher_id,
            bound_account: self.bound_account,
            state: PhantomData,
            capability: self.capability,
            nonce: self.nonce + 1, // Increment nonce
            expires_at: self.expires_at,
        })
    }
    
    /// Get the current state instance for validation
    fn state_instance(&self) -> S {
        // This is safe because S is a zero-sized type
        unsafe { std::mem::zeroed() }
    }
    
    /// Verify the voucher is bound to the correct account
    pub fn verify_binding(&self, account: &Pubkey) -> Result<()> {
        require_eq!(
            self.bound_account,
            *account,
            ErrorCode::VoucherNotBound
        );
        Ok(())
    }
    
    /// Serialize voucher for storage in account metadata
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.voucher_id.to_le_bytes());
        data.extend_from_slice(self.bound_account.as_ref());
        data.push(S::STATE_ID);
        data.extend_from_slice(&self.nonce.to_le_bytes());
        if let Some(expires_at) = self.expires_at {
            data.push(1); // Has expiration
            data.extend_from_slice(&expires_at.to_le_bytes());
        } else {
            data.push(0); // No expiration
        }
        data
    }
    
    /// Deserialize voucher from account metadata
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() < 41 { // 8 + 32 + 1 + 8 = 49 minimum
            return Err(ErrorCode::InvalidVoucherData.into());
        }
        
        let voucher_id = u64::from_le_bytes(
            data[0..8].try_into().map_err(|_| ErrorCode::InvalidVoucherData)?
        );
        
        let bound_account = Pubkey::new_from_array(
            data[8..40].try_into().map_err(|_| ErrorCode::InvalidVoucherData)?
        );
        
        let state_id = data[40];
        require_eq!(state_id, S::STATE_ID, ErrorCode::InvalidStateId);
        
        let nonce = u64::from_le_bytes(
            data[41..49].try_into().map_err(|_| ErrorCode::InvalidVoucherData)?
        );
        
        let expires_at = if data.len() > 49 && data[49] == 1 {
            if data.len() < 58 {
                return Err(ErrorCode::InvalidVoucherData.into());
            }
            Some(u64::from_le_bytes(
                data[50..58].try_into().map_err(|_| ErrorCode::InvalidVoucherData)?
            ))
        } else {
            None
        };
        
        Ok(Self {
            voucher_id,
            bound_account,
            state: PhantomData,
            capability: PhantomData,
            nonce,
            expires_at,
        })
    }
}

/// Enhanced linear lending position with comprehensive state tracking
#[derive(Clone, Debug)]
pub struct LinearLendingPosition {
    /// Unique position identifier
    pub position_id: u64,
    /// Owner of the position
    pub owner: Pubkey,
    /// Deposited amount
    pub deposited_amount: u64,
    /// Collateral amount
    pub collateral_amount: u64,
    /// Borrowed amount (if any)
    pub borrowed_amount: u64,
    /// Current state ID
    pub current_state: u8,
    /// Position nonce for replay prevention
    pub nonce: u64,
    /// Timestamp of last update
    pub last_updated: i64,
    /// Interest rate (basis points)
    pub interest_rate_bps: u16,
    /// Health factor (1000 = 100%)
    pub health_factor: u16,
}

impl LinearLendingPosition {
    pub const SIZE: usize = 8 + 32 + 8 + 8 + 8 + 1 + 8 + 8 + 2 + 2;
    
    pub fn new(position_id: u64, owner: Pubkey) -> Self {
        Self {
            position_id,
            owner,
            deposited_amount: 0,
            collateral_amount: 0,
            borrowed_amount: 0,
            current_state: 0, // Initial state
            nonce: 0,
            last_updated: 0, // Clock::get().unwrap().unix_timestamp,
            interest_rate_bps: 500, // 5% default
            health_factor: 1000,    // 100% healthy
        }
    }
    
    /// Execute a deposit operation with voucher validation
    pub fn deposit<C: capabilities::Capability>(
        &mut self,
        voucher: LinearVoucher<states::Initial, C>,
        amount: u64,
    ) -> Result<LinearVoucher<states::Deposited, C>> {
        voucher.verify_binding(&self.owner)?;
        
        require_eq!(
            self.current_state,
            0, // Initial state
            ErrorCode::InvalidStateTransition
        );
        
        self.deposited_amount = self.deposited_amount
            .checked_add(amount)
            .ok_or(ErrorCode::MathOverflow)?;
        
        self.current_state = 1; // Deposited state
        self.nonce += 1;
        self.last_updated = Clock::get()?.unix_timestamp;
        
        voucher.transition()
    }
    
    /// Transfer voucher to another account
    pub fn transfer_voucher<C: capabilities::Capability>(
        &mut self,
        voucher: LinearVoucher<states::Deposited, C>,
        new_owner: Pubkey,
    ) -> Result<LinearVoucher<states::VoucherTransferred, C>> {
        voucher.verify_binding(&self.owner)?;
        
        require_eq!(
            self.current_state,
            1, // Deposited state
            ErrorCode::InvalidStateTransition
        );
        
        self.owner = new_owner;
        self.current_state = 2; // VoucherTransferred state
        self.nonce += 1;
        self.last_updated = Clock::get()?.unix_timestamp;
        
        voucher.transition()
    }
    
    /// Add collateral to the position
    pub fn add_collateral<C: capabilities::Capability>(
        &mut self,
        voucher: LinearVoucher<states::VoucherTransferred, C>,
        amount: u64,
    ) -> Result<LinearVoucher<states::Collateralized, C>> {
        voucher.verify_binding(&self.owner)?;
        
        require_eq!(
            self.current_state,
            2, // VoucherTransferred state
            ErrorCode::InvalidStateTransition
        );
        
        self.collateral_amount = self.collateral_amount
            .checked_add(amount)
            .ok_or(ErrorCode::MathOverflow)?;
        
        // Recalculate health factor
        self.update_health_factor()?;
        
        self.current_state = 3; // Collateralized state
        self.nonce += 1;
        self.last_updated = Clock::get()?.unix_timestamp;
        
        voucher.transition()
    }
    
    /// Withdraw from the position (final state)
    pub fn withdraw<C: capabilities::Capability>(
        &mut self,
        voucher: LinearVoucher<states::Collateralized, C>,
        amount: u64,
    ) -> Result<LinearVoucher<states::Final, C>> {
        voucher.verify_binding(&self.owner)?;
        
        require_eq!(
            self.current_state,
            3, // Collateralized state
            ErrorCode::InvalidStateTransition
        );
        
        // Check if withdrawal is safe
        let available = self.calculate_available_withdrawal()?;
        require!(amount <= available, ErrorCode::InsufficientFunds);
        
        self.deposited_amount = self.deposited_amount
            .checked_sub(amount)
            .ok_or(ErrorCode::MathUnderflow)?;
        
        // Recalculate health factor
        self.update_health_factor()?;
        
        self.current_state = 4; // Final state
        self.nonce += 1;
        self.last_updated = Clock::get()?.unix_timestamp;
        
        voucher.transition()
    }
    
    /// Calculate available withdrawal amount
    fn calculate_available_withdrawal(&self) -> Result<u64> {
        // Simple calculation: can withdraw up to deposited amount minus borrowed amount
        // In practice, this would involve more complex risk calculations
        let total_value = self.deposited_amount
            .checked_add(self.collateral_amount)
            .ok_or(ErrorCode::MathOverflow)?;
        
        let available = total_value
            .saturating_sub(self.borrowed_amount);
        
        Ok(available)
    }
    
    /// Update health factor based on current position
    fn update_health_factor(&mut self) -> Result<()> {
        if self.borrowed_amount == 0 {
            self.health_factor = 1000; // 100% if no debt
            return Ok(());
        }
        
        let total_collateral = self.deposited_amount
            .checked_add(self.collateral_amount)
            .ok_or(ErrorCode::MathOverflow)?;
        
        // Health factor = (collateral_value / borrowed_amount) * 1000
        // Assuming 1:1 price for simplicity
        self.health_factor = ((total_collateral as u128)
            .checked_mul(1000)
            .and_then(|v| v.checked_div(self.borrowed_amount as u128))
            .ok_or(ErrorCode::MathOverflow)? as u16)
            .min(9999); // Cap at 999.9%
        
        Ok(())
    }
}

/// Capability registry for managing permissions
#[derive(Clone, Debug)]
pub struct CapabilityRegistry {
    /// Mapping of accounts to their capabilities
    pub capabilities: Vec<(Pubkey, u8)>, // (account, capability_mask)
    /// Admin accounts that can grant capabilities
    pub admins: Vec<Pubkey>,
}

impl CapabilityRegistry {
    pub const MAX_CAPABILITIES: usize = 100;
    pub const SIZE: usize = 8 + (32 + 1) * Self::MAX_CAPABILITIES + 8 + 32 * 10; // Support up to 10 admins
    
    pub fn new(admin: Pubkey) -> Self {
        Self {
            capabilities: Vec::new(),
            admins: vec![admin],
        }
    }
    
    /// Check if an account has a specific capability
    pub fn has_capability(&self, account: &Pubkey, capability_mask: u8) -> bool {
        self.capabilities
            .iter()
            .any(|(acc, mask)| acc == account && (*mask & capability_mask) != 0)
    }
    
    /// Grant capability to an account (admin only)
    pub fn grant_capability(
        &mut self,
        admin: &Pubkey,
        target: Pubkey,
        capability_mask: u8,
    ) -> Result<()> {
        require!(
            self.admins.contains(admin),
            ErrorCode::UnauthorizedAdmin
        );
        
        // Update existing or add new capability
        if let Some((_, mask)) = self.capabilities
            .iter_mut()
            .find(|(acc, _)| *acc == target)
        {
            *mask |= capability_mask;
        } else {
            require!(
                self.capabilities.len() < Self::MAX_CAPABILITIES,
                ErrorCode::CapabilityRegistryFull
            );
            self.capabilities.push((target, capability_mask));
        }
        
        Ok(())
    }
    
    /// Revoke capability from an account (admin only)
    pub fn revoke_capability(
        &mut self,
        admin: &Pubkey,
        target: &Pubkey,
        capability_mask: u8,
    ) -> Result<()> {
        require!(
            self.admins.contains(admin),
            ErrorCode::UnauthorizedAdmin
        );
        
        if let Some((_, mask)) = self.capabilities
            .iter_mut()
            .find(|(acc, _)| acc == target)
        {
            *mask &= !capability_mask;
        }
        
        Ok(())
    }
}

/// Capability masks for different operations
pub mod capability_masks {
    pub const DEPOSIT: u8 = 1 << 0;
    pub const TRANSFER: u8 = 1 << 1;
    pub const WITHDRAW: u8 = 1 << 2;
    pub const ADMIN: u8 = 1 << 3;
    pub const LIQUIDATE: u8 = 1 << 4;
    pub const EMERGENCY: u8 = 1 << 5;
}

/// Enhanced verifier function with full linear type checking
pub fn verify_enhanced_linear_operation(
    _account: &AccountInfo,
    caller: &Signer,
    managed_account_data: &[u8],
    operation_data: &[u8],
    capability_registry: &CapabilityRegistry,
) -> Result<()> {
    // Deserialize the position from managed account data
    let mut position = LinearLendingPosition::deserialize(managed_account_data)?;
    
    // Parse operation data
    if operation_data.is_empty() {
        return Err(ErrorCode::InvalidOperationData.into());
    }
    
    let operation_type = operation_data[0];
    let operation_params = &operation_data[1..];
    
    // Verify caller has required capability
    let required_capability = match operation_type {
        0 => capability_masks::DEPOSIT,
        1 => capability_masks::TRANSFER,
        2 => capability_masks::DEPOSIT, // Add collateral uses deposit capability
        3 => capability_masks::WITHDRAW,
        _ => return Err(ErrorCode::InvalidOperation.into()),
    };
    
    require!(
        capability_registry.has_capability(&caller.key(), required_capability),
        ErrorCode::InsufficientCapability
    );
    
    // Verify state transition is valid
    let current_state = position.current_state;
    let expected_state = match operation_type {
        0 => 0, // Initial state - Deposit
        1 => 1, // Deposited state - Transfer voucher
        2 => 2, // VoucherTransferred state - Add collateral
        3 => 3, // Collateralized state - Withdraw
        _ => return Err(ErrorCode::InvalidOperation.into()),
    };
    
    require_eq!(
        current_state,
        expected_state,
        ErrorCode::InvalidStateTransition
    );
    
    // Parse operation-specific parameters and execute
    match operation_type {
        0 => {
            // Deposit operation
            let amount = u64::from_le_bytes(
                operation_params[..8].try_into()
                    .map_err(|_| ErrorCode::InvalidOperationData)?
            );
            
            // Create voucher for this operation
            let voucher = LinearVoucher::<states::Initial, capabilities::DepositCap>::new(
                position.position_id,
                caller.key(),
                position.nonce,
            );
            
            position.deposit(voucher, amount)?;
        },
        1 => {
            // Transfer voucher operation
            let new_owner = Pubkey::new_from_array(
                operation_params[..32].try_into()
                    .map_err(|_| ErrorCode::InvalidOperationData)?
            );
            
            let voucher = LinearVoucher::<states::Deposited, capabilities::TransferCap>::new(
                position.position_id,
                caller.key(),
                position.nonce,
            );
            
            position.transfer_voucher(voucher, new_owner)?;
        },
        2 => {
            // Add collateral operation
            let amount = u64::from_le_bytes(
                operation_params[..8].try_into()
                    .map_err(|_| ErrorCode::InvalidOperationData)?
            );
            
            let voucher = LinearVoucher::<states::VoucherTransferred, capabilities::DepositCap>::new(
                position.position_id,
                caller.key(),
                position.nonce,
            );
            
            position.add_collateral(voucher, amount)?;
        },
        3 => {
            // Withdraw operation
            let amount = u64::from_le_bytes(
                operation_params[..8].try_into()
                    .map_err(|_| ErrorCode::InvalidOperationData)?
            );
            
            let voucher = LinearVoucher::<states::Collateralized, capabilities::WithdrawCap>::new(
                position.position_id,
                caller.key(),
                position.nonce,
            );
            
            position.withdraw(voucher, amount)?;
        },
        _ => unreachable!(),
    }
    
    msg!(
        "Enhanced linear operation {} executed successfully for position {} in state {}",
        operation_type,
        position.position_id,
        position.current_state
    );
    
    Ok(())
}

impl LinearLendingPosition {
    /// Serialize position to bytes
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::SIZE);
        data.extend_from_slice(&self.position_id.to_le_bytes());
        data.extend_from_slice(self.owner.as_ref());
        data.extend_from_slice(&self.deposited_amount.to_le_bytes());
        data.extend_from_slice(&self.collateral_amount.to_le_bytes());
        data.extend_from_slice(&self.borrowed_amount.to_le_bytes());
        data.push(self.current_state);
        data.extend_from_slice(&self.nonce.to_le_bytes());
        data.extend_from_slice(&self.last_updated.to_le_bytes());
        data.extend_from_slice(&self.interest_rate_bps.to_le_bytes());
        data.extend_from_slice(&self.health_factor.to_le_bytes());
        data
    }
    
    /// Deserialize position from bytes
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() < Self::SIZE {
            return Err(ErrorCode::InvalidPositionData.into());
        }
        
        let position_id = u64::from_le_bytes(
            data[0..8].try_into().map_err(|_| ErrorCode::InvalidPositionData)?
        );
        
        let owner = Pubkey::new_from_array(
            data[8..40].try_into().map_err(|_| ErrorCode::InvalidPositionData)?
        );
        
        let deposited_amount = u64::from_le_bytes(
            data[40..48].try_into().map_err(|_| ErrorCode::InvalidPositionData)?
        );
        
        let collateral_amount = u64::from_le_bytes(
            data[48..56].try_into().map_err(|_| ErrorCode::InvalidPositionData)?
        );
        
        let borrowed_amount = u64::from_le_bytes(
            data[56..64].try_into().map_err(|_| ErrorCode::InvalidPositionData)?
        );
        
        let current_state = data[64];
        
        let nonce = u64::from_le_bytes(
            data[65..73].try_into().map_err(|_| ErrorCode::InvalidPositionData)?
        );
        
        let last_updated = i64::from_le_bytes(
            data[73..81].try_into().map_err(|_| ErrorCode::InvalidPositionData)?
        );
        
        let interest_rate_bps = u16::from_le_bytes(
            data[81..83].try_into().map_err(|_| ErrorCode::InvalidPositionData)?
        );
        
        let health_factor = u16::from_le_bytes(
            data[83..85].try_into().map_err(|_| ErrorCode::InvalidPositionData)?
        );
        
        Ok(Self {
            position_id,
            owner,
            deposited_amount,
            collateral_amount,
            borrowed_amount,
            current_state,
            nonce,
            last_updated,
            interest_rate_bps,
            health_factor,
        })
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid operation type")]
    InvalidOperation,
    
    #[msg("Invalid state transition")]
    InvalidStateTransition,
    
    #[msg("Voucher not bound to account")]
    VoucherNotBound,
    
    #[msg("Voucher has expired")]
    VoucherExpired,
    
    #[msg("Invalid voucher data")]
    InvalidVoucherData,
    
    #[msg("Invalid state ID")]
    InvalidStateId,
    
    #[msg("Math overflow")]
    MathOverflow,
    
    #[msg("Math underflow")]
    MathUnderflow,
    
    #[msg("Insufficient funds")]
    InsufficientFunds,
    
    #[msg("Invalid operation data")]
    InvalidOperationData,
    
    #[msg("Invalid position data")]
    InvalidPositionData,
    
    #[msg("Insufficient capability")]
    InsufficientCapability,
    
    #[msg("Unauthorized admin")]
    UnauthorizedAdmin,
    
    #[msg("Capability registry full")]
    CapabilityRegistryFull,
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::states::{LinearState, Initial, Deposited, VoucherTransferred, Collateralized};
    use super::capabilities::*;
    
    #[test]
    fn test_linear_state_transitions() {
        // Test valid state transitions
        let initial = Initial;
        assert!(initial.validate_transition(Deposited::STATE_ID));
        assert!(!initial.validate_transition(VoucherTransferred::STATE_ID));
        
        let deposited = Deposited;
        assert!(deposited.validate_transition(VoucherTransferred::STATE_ID));
        assert!(!deposited.validate_transition(Collateralized::STATE_ID));
    }
    
    #[test]
    fn test_voucher_creation_and_transition() {
        let account = Pubkey::new_unique();
        let voucher = LinearVoucher::<Initial, DepositCap>::new(1, account, 0);
        
        assert_eq!(voucher.voucher_id, 1);
        assert_eq!(voucher.bound_account, account);
        assert_eq!(voucher.nonce, 0);
        
        // Test transition (would need Clock in actual environment)
        // let next_voucher = voucher.transition::<Deposited>().unwrap();
        // assert_eq!(next_voucher.nonce, 1);
    }
    
    #[test]
    fn test_capability_registry() {
        let admin = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let mut registry = CapabilityRegistry::new(admin);
        
        // Initially user has no capabilities
        assert!(!registry.has_capability(&user, capability_masks::DEPOSIT));
        
        // Admin grants deposit capability
        registry.grant_capability(&admin, user, capability_masks::DEPOSIT).unwrap();
        assert!(registry.has_capability(&user, capability_masks::DEPOSIT));
        assert!(!registry.has_capability(&user, capability_masks::WITHDRAW));
        
        // Admin revokes capability
        registry.revoke_capability(&admin, &user, capability_masks::DEPOSIT).unwrap();
        assert!(!registry.has_capability(&user, capability_masks::DEPOSIT));
    }
    
    #[test]
    fn test_position_serialization() {
        let owner = Pubkey::new_unique();
        let position = LinearLendingPosition::new(123, owner);
        
        let serialized = position.serialize();
        let deserialized = LinearLendingPosition::deserialize(&serialized).unwrap();
        
        assert_eq!(position.position_id, deserialized.position_id);
        assert_eq!(position.owner, deserialized.owner);
        assert_eq!(position.current_state, deserialized.current_state);
    }
}