// Global CPI allowlist for valence-kernel secure program invocation control
//
// The valence-kernel maintains a global allowlist of programs that can be invoked
// through CPI calls, providing a security boundary that prevents execution of
// unauthorized programs while enabling controlled access to verified external
// functionality through the kernel's execution engine.
//
// KERNEL INTEGRATION: Before executing any CPI operation, the batch engine consults
// the global allowlist to verify that the target program is authorized. This provides
// a crucial security layer that prevents malicious or unverified programs from being
// executed through kernel operations.
//
// SECURITY MODEL: The allowlist implements a strict whitelist approach where only
// explicitly approved programs can be called, ensuring that the kernel cannot be
// used as a vector for executing malicious code while still enabling integration
// with trusted external protocols and functions.
use anchor_lang::prelude::*;
use std::str::FromStr;
use crate::errors::KernelError;

// ================================
// Allowlist Account
// ================================

/// Global allowlist of programs that can be invoked via CPI
#[account]
pub struct AllowlistAccount {
    /// Authority that can modify the allowlist
    pub authority: Pubkey,
    /// Fixed-size array of allowed program IDs
    pub allowed_programs: [Pubkey; 32],
    /// Number of active programs in the array
    pub program_count: u8,
    /// Version for future upgrades
    pub version: u8,
}

impl AllowlistAccount {
    pub const MAX_PROGRAMS: usize = 32;
    
    /// Calculate space needed for allowlist
    pub fn space() -> usize {
        8 + // discriminator
        32 + // authority
        32 * Self::MAX_PROGRAMS + // allowed_programs fixed array
        1 + // program_count
        1 // version
    }
    
    /// Initialize new allowlist
    #[must_use]
    pub fn new(authority: Pubkey) -> Self {
        Self {
            authority,
            allowed_programs: [Pubkey::default(); 32],
            program_count: 0,
            version: 1,
        }
    }
    
    /// Check if a program is allowed
    pub fn is_allowed(&self, program_id: &Pubkey) -> bool {
        // System program and SPL programs are always allowed
        if is_solana_system_program(program_id) || is_solana_spl_program(program_id) {
            return true;
        }
        
        self.allowed_programs[..self.program_count as usize]
            .iter()
            .any(|p| p == program_id)
    }
    
    /// Add a program to the allowlist
    pub fn add_program(&mut self, program_id: Pubkey) -> Result<()> {
        require!(
            (self.program_count as usize) < Self::MAX_PROGRAMS,
            KernelError::AllowlistFull
        );
        
        require!(
            !self.is_allowed(&program_id),
            KernelError::ProgramAlreadyAllowed
        );
        
        self.allowed_programs[self.program_count as usize] = program_id;
        self.program_count += 1;
        Ok(())
    }
    
    /// Remove a program from the allowlist
    pub fn remove_program(&mut self, program_id: &Pubkey) -> Result<()> {
        let active_slice = &mut self.allowed_programs[..self.program_count as usize];
        let index = active_slice
            .iter()
            .position(|p| p == program_id)
            .ok_or(KernelError::ProgramNotAllowed)?;
        
        // Move the last element to the removed position
        if index < (self.program_count as usize - 1) {
            self.allowed_programs[index] = self.allowed_programs[self.program_count as usize - 1];
        }
        
        // Clear the last position and decrement count
        self.allowed_programs[self.program_count as usize - 1] = Pubkey::default();
        self.program_count -= 1;
        Ok(())
    }
}

/// Check if program is a system program
fn is_solana_system_program(program_id: &Pubkey) -> bool {
    program_id == &anchor_lang::system_program::ID ||
    program_id == &anchor_lang::solana_program::sysvar::clock::ID ||
    program_id == &anchor_lang::solana_program::sysvar::rent::ID
}

/// Check if program is an SPL program
fn is_solana_spl_program(program_id: &Pubkey) -> bool {
    // Common SPL programs
    let spl_token = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").ok();
    let spl_token_2022 = Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb").ok();
    let spl_associated_token = Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").ok();
    
    Some(program_id) == spl_token.as_ref() ||
    Some(program_id) == spl_token_2022.as_ref() ||
    Some(program_id) == spl_associated_token.as_ref()
}