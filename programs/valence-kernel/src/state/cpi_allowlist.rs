// CPI allowlist for secure program invocation
use anchor_lang::prelude::*;
use std::str::FromStr;
use crate::errors::KernelError;

// ================================
// CPI Allowlist Account
// ================================

/// Global allowlist of programs that can be invoked via CPI
#[account]
pub struct CpiAllowlistAccount {
    /// Authority that can modify the allowlist
    pub authority: Pubkey,
    /// List of allowed program IDs
    pub allowed_programs: Vec<Pubkey>,
    /// Version for future upgrades
    pub version: u8,
}

impl CpiAllowlistAccount {
    pub const MAX_PROGRAMS: usize = 32;
    
    /// Calculate space needed for allowlist
    pub fn space() -> usize {
        8 + // discriminator
        32 + // authority
        4 + (32 * Self::MAX_PROGRAMS) + // allowed_programs vec
        1 // version
    }
    
    /// Initialize new allowlist
    #[must_use]
    pub fn new(authority: Pubkey) -> Self {
        Self {
            authority,
            allowed_programs: Vec::with_capacity(Self::MAX_PROGRAMS),
            version: 1,
        }
    }
    
    /// Check if a program is allowed
    pub fn is_allowed(&self, program_id: &Pubkey) -> bool {
        // System program and SPL programs are always allowed
        if is_solana_system_program(program_id) || is_solana_spl_program(program_id) {
            return true;
        }
        
        self.allowed_programs.contains(program_id)
    }
    
    /// Add a program to the allowlist
    pub fn add_program(&mut self, program_id: Pubkey) -> Result<()> {
        require!(
            self.allowed_programs.len() < Self::MAX_PROGRAMS,
            KernelError::AllowlistFull
        );
        
        require!(
            !self.allowed_programs.contains(&program_id),
            KernelError::ProgramAlreadyAllowed
        );
        
        self.allowed_programs.push(program_id);
        Ok(())
    }
    
    /// Remove a program from the allowlist
    pub fn remove_program(&mut self, program_id: &Pubkey) -> Result<()> {
        let index = self.allowed_programs
            .iter()
            .position(|p| p == program_id)
            .ok_or(KernelError::ProgramNotAllowed)?;
            
        self.allowed_programs.swap_remove(index);
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