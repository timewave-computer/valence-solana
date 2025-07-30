// Account lookup table for valence-kernel session account pre-registration
//
// The valence-kernel eliminates Solana's fragile remaining_accounts pattern by
// requiring all accounts to be pre-registered in session-specific lookup tables.
// This provides security through explicit account declarations while enabling
// efficient index-based references in operation batches.
//
// KERNEL INTEGRATION: The batch execution engine uses account lookup tables to
// resolve account indices to actual addresses, validate permissions, and enforce
// borrowing semantics. This eliminates the need for clients to manage complex
// account ordering and provides strong security guarantees.
//
// SECURITY MODEL: Pre-registration ensures that sessions can only access explicitly
// declared accounts with specified permissions, preventing unauthorized account
// access and providing clear security boundaries for operation execution.

use anchor_lang::prelude::*;
use crate::errors::KernelError;
use crate::MAX_REGISTERED_ACCOUNTS;

/// Session-specific account lookup table
/// 
/// This account stores pre-validated account addresses that can be used
/// by the session during operation execution. Accounts must be registered
/// before use, providing on-chain validation of addresses and permissions.
#[account]
#[derive(Debug)]
pub struct SessionAccountLookup {
    /// The session this lookup table belongs to
    pub session: Pubkey,
    
    /// Authority that can modify the lookup table
    pub authority: Pubkey,
    
    /// Fixed-size array of borrowable accounts
    pub borrowable_accounts: [RegisteredAccount; MAX_REGISTERED_ACCOUNTS],
    /// Number of active borrowable accounts
    pub borrowable_count: u8,
    
    /// Fixed-size array of programs for CPI
    pub program_accounts: [RegisteredProgram; MAX_REGISTERED_ACCOUNTS],
    /// Number of active programs
    pub program_count: u8,
    
    /// Fixed-size array of guard accounts
    pub guard_accounts: [RegisteredAccount; MAX_REGISTERED_ACCOUNTS],
    /// Number of active guard accounts
    pub guard_count: u8,
    
    /// Version for future upgrades
    pub version: u8,
}

/// A registered account with metadata
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RegisteredAccount {
    /// The account's public key
    pub address: Pubkey,
    
    /// Permissions bitmap (read, write, etc.)
    pub permissions: u8,
    
    /// Optional label for debugging
    pub label: [u8; 32],
}

/// A registered program for CPI
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RegisteredProgram {
    /// The program's public key
    pub address: Pubkey,
    
    /// Whether this program is currently active
    pub active: bool,
    
    /// Optional label for debugging
    pub label: [u8; 32],
}

impl SessionAccountLookup {
    /// Calculate space needed for a lookup table
    pub fn calculate_space() -> usize {
        8 + // discriminator
        32 + // session
        32 + // authority
        (MAX_REGISTERED_ACCOUNTS * RegisteredAccount::SIZE) + // borrowable_accounts
        1 + // borrowable_count
        (MAX_REGISTERED_ACCOUNTS * RegisteredProgram::SIZE) + // program_accounts
        1 + // program_count
        (MAX_REGISTERED_ACCOUNTS * RegisteredAccount::SIZE) + // guard_accounts
        1 + // guard_count
        1 // version
    }
    
    /// Create a new lookup table
    pub fn new(session: Pubkey, authority: Pubkey) -> Self {
        const DEFAULT_ACCOUNT: RegisteredAccount = RegisteredAccount {
            address: Pubkey::new_from_array([0u8; 32]),
            permissions: 0,
            label: [0u8; 32],
        };
        const DEFAULT_PROGRAM: RegisteredProgram = RegisteredProgram {
            address: Pubkey::new_from_array([0u8; 32]),
            active: false,
            label: [0u8; 32],
        };
        
        Self {
            session,
            authority,
            borrowable_accounts: [DEFAULT_ACCOUNT; MAX_REGISTERED_ACCOUNTS],
            borrowable_count: 0,
            program_accounts: [DEFAULT_PROGRAM; MAX_REGISTERED_ACCOUNTS],
            program_count: 0,
            guard_accounts: [DEFAULT_ACCOUNT; MAX_REGISTERED_ACCOUNTS],
            guard_count: 0,
            version: 1,
        }
    }
    
    /// Register a borrowable account
    pub fn register_borrowable(
        &mut self,
        address: Pubkey,
        permissions: u8,
        label: [u8; 32],
    ) -> Result<()> {
        require!(
            (self.borrowable_count as usize) < MAX_REGISTERED_ACCOUNTS,
            KernelError::TooManyAccounts
        );
        
        // Check for duplicates
        let active_slice = &self.borrowable_accounts[..self.borrowable_count as usize];
        require!(
            !active_slice.iter().any(|a| a.address == address),
            KernelError::DuplicateAccount
        );
        
        self.borrowable_accounts[self.borrowable_count as usize] = RegisteredAccount {
            address,
            permissions,
            label,
        };
        self.borrowable_count += 1;
        
        Ok(())
    }
    
    /// Register a program for CPI
    pub fn register_program(
        &mut self,
        address: Pubkey,
        label: [u8; 32],
    ) -> Result<()> {
        require!(
            (self.program_count as usize) < MAX_REGISTERED_ACCOUNTS,
            KernelError::TooManyAccounts
        );
        
        // Check for duplicates
        let active_slice = &self.program_accounts[..self.program_count as usize];
        require!(
            !active_slice.iter().any(|p| p.address == address),
            KernelError::DuplicateAccount
        );
        
        self.program_accounts[self.program_count as usize] = RegisteredProgram {
            address,
            active: true,
            label,
        };
        self.program_count += 1;
        
        Ok(())
    }
    
    /// Register a guard account
    pub fn register_guard(
        &mut self,
        address: Pubkey,
        permissions: u8,
        label: [u8; 32],
    ) -> Result<()> {
        require!(
            (self.guard_count as usize) < MAX_REGISTERED_ACCOUNTS,
            KernelError::TooManyAccounts
        );
        
        // Check for duplicates
        let active_slice = &self.guard_accounts[..self.guard_count as usize];
        require!(
            !active_slice.iter().any(|a| a.address == address),
            KernelError::DuplicateAccount
        );
        
        self.guard_accounts[self.guard_count as usize] = RegisteredAccount {
            address,
            permissions,
            label,
        };
        self.guard_count += 1;
        
        Ok(())
    }
    
    /// Get a borrowable account by index
    pub fn get_borrowable(&self, index: usize) -> Result<&RegisteredAccount> {
        require!(
            index < self.borrowable_count as usize,
            KernelError::AccountIndexOutOfBounds
        );
        Ok(&self.borrowable_accounts[index])
    }
    
    /// Get a program by index
    pub fn get_program(&self, index: usize) -> Result<&RegisteredProgram> {
        require!(
            index < self.program_count as usize,
            KernelError::InvalidProgramIndex
        );
        Ok(&self.program_accounts[index])
    }
    
    /// Find index of a borrowable account
    pub fn find_borrowable_index(&self, address: &Pubkey) -> Option<usize> {
        self.borrowable_accounts[..self.borrowable_count as usize]
            .iter()
            .position(|a| a.address == *address)
    }
    
    /// Validate that an account is registered as borrowable
    pub fn validate_borrowable(&self, address: &Pubkey, required_permissions: u8) -> Result<()> {
        let account = self.borrowable_accounts[..self.borrowable_count as usize]
            .iter()
            .find(|a| a.address == *address)
            .ok_or(KernelError::UnregisteredAccount)?;
            
        require!(
            account.permissions & required_permissions == required_permissions,
            KernelError::InsufficientPermissions
        );
        
        Ok(())
    }
}

impl RegisteredAccount {
    pub const SIZE: usize = 32 + 1 + 32; // address + permissions + label
}

impl RegisteredProgram {
    pub const SIZE: usize = 32 + 1 + 32; // address + active + label
}