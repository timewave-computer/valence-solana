// Flattened guard instruction set for high-performance evaluation
// Provides a minimal, non-recursive bytecode for guard logic
use crate::validation;
use anchor_lang::prelude::*;

// ================================
// Guard Opcodes
// ================================

/// Minimal instruction set for guard evaluation
/// Authorization Processing Unit (APU) - returns true or false
#[derive(Clone, Debug, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub enum GuardOp {
    // ===== State Assertion Opcodes =====
    /// Sets result_flag to true if caller is session owner
    CheckOwner,
    /// Sets result_flag to true if clock.unix_timestamp < timestamp
    CheckExpiry { timestamp: i64 },
    /// Sets result_flag to true if clock.unix_timestamp >= timestamp
    CheckNotBefore { timestamp: i64 },
    /// Sets result_flag to true if session.usage_count < limit
    CheckUsageLimit { limit: u64 },
    
    // ===== Control Flow Opcodes =====
    /// If result_flag is false, adds offset to instruction pointer
    JumpIfFalse { offset: u8 },
    /// Halts execution and returns true (successful exit)
    Terminate,
    /// Halts execution and returns false
    Abort,
    
    // ===== External Interaction Opcode =====
    /// Executes CPI at instruction_manifest[manifest_index]
    Invoke { manifest_index: u8 },
}

impl GuardOp {
    /// Get the size of this opcode in bytes
    pub fn size(&self) -> usize {
        match self {
            GuardOp::CheckOwner => 1,
            GuardOp::CheckExpiry { .. } | GuardOp::CheckNotBefore { .. } => 1 + 8,
            GuardOp::CheckUsageLimit { .. } => 1 + 8,
            GuardOp::JumpIfFalse { .. } => 1 + 1,
            GuardOp::Terminate | GuardOp::Abort => 1,
            GuardOp::Invoke { .. } => 1 + 1,
        }
    }
}

// ================================
// CPI Manifest Entry
// ================================

/// Represents a CPI instruction that can be invoked by guards
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct CPIManifestEntry {
    /// Program to invoke
    pub program_id: Pubkey,
    /// Required accounts (indices into remaining_accounts)
    pub account_indices: Vec<u8>,
    /// Instruction data
    pub data: Vec<u8>,
}

// ================================
// Compiled Guard Program
// ================================

/// A compiled guard program ready for execution
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct CompiledGuard {
    /// Flattened opcodes
    pub opcodes: Vec<GuardOp>,
    /// CPI instructions that can be invoked
    pub cpi_manifest: Vec<CPIManifestEntry>,
}

impl CompiledGuard {
    /// Calculate space needed for this compiled guard
    pub fn space(&self) -> usize {
        8 + // discriminator
        4 + // opcodes vec length
        self.opcodes.iter().map(|op| op.size()).sum::<usize>() +
        4 + // cpi_manifest vec length
        self.cpi_manifest.iter()
            .map(|entry| {
                32 + // program_id
                4 + entry.account_indices.len() + // indices vec
                4 + entry.data.len() // data vec
            })
            .sum::<usize>()
    }
    
    /// Validate the compiled guard
    pub fn validate(&self) -> Result<()> {
        // Check opcode count
        require!(
            self.opcodes.len() <= 255,
            crate::errors::ValenceError::GuardDataTooLarge
        );
        
        // Check manifest size
        require!(
            self.cpi_manifest.len() <= 16,
            crate::errors::ValenceError::GuardDataTooLarge
        );
        
        // Validate each CPI manifest entry
        for entry in &self.cpi_manifest {
            validation::validate_cpi_data(&entry.data)?;
            validation::validate_account_indices(&entry.account_indices, 15)?;
        }
        
        // Validate jumps
        for (i, op) in self.opcodes.iter().enumerate() {
            match op {
                GuardOp::JumpIfFalse { offset } => {
                    let target = i + *offset as usize;
                    require!(
                        target < self.opcodes.len(),
                        crate::errors::ValenceError::InvalidParameters
                    );
                }
                GuardOp::Invoke { manifest_index } => {
                    require!(
                        (*manifest_index as usize) < self.cpi_manifest.len(),
                        crate::errors::ValenceError::InvalidParameters
                    );
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// Create a new compiled guard
    pub fn new(opcodes: Vec<GuardOp>) -> Result<Self> {
        let guard = Self {
            opcodes,
            cpi_manifest: Vec::new(),
        };
        guard.validate()?;
        Ok(guard)
    }
    
    /// Create a new compiled guard with CPI manifest
    pub fn new_with_manifest(opcodes: Vec<GuardOp>, cpi_manifest: Vec<CPIManifestEntry>) -> Result<Self> {
        let guard = Self {
            opcodes,
            cpi_manifest,
        };
        guard.validate()?;
        Ok(guard)
    }
}