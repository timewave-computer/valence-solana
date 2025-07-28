// Guard opcode evaluator - executes flattened guard programs
// Provides a safe, non-recursive evaluation engine
use super::opcodes::{GuardOp, SerializedGuard};
use crate::errors::KernelError;
use anchor_lang::prelude::*;
use anchor_lang::solana_program;

// ================================
// Evaluation Context
// ================================

/// Context for guard evaluation
pub struct GuardEvaluationContext<'a> {
    /// The session being evaluated
    pub session: &'a crate::state::Session,
    /// The caller attempting access
    pub caller: &'a Pubkey,
    /// Current clock state
    pub clock: &'a Clock,
    /// The operation being performed
    pub operation: &'a [u8],
    /// Additional accounts for lookups
    pub remaining_accounts: &'a [AccountInfo<'a>],
}

// ===================================
// Authorization Processing Unit (APU)
// ===================================

/// Evaluate a compiled guard program
/// Returns true or false based on authorization logic
pub fn execute_serialized_guard(
    guard: &SerializedGuard,
    ctx: &GuardEvaluationContext,
) -> Result<bool> {
    // Validate guard before execution
    guard.validate()?;
    
    // Initialize APU state
    let mut result_flag = false;
    let mut instruction_pointer = 0;
    
    // Execute opcodes
    while instruction_pointer < guard.opcodes.len() {
        let op = &guard.opcodes[instruction_pointer];
        
        match op {
            // ===== State Assertion Opcodes =====
            GuardOp::CheckOwner => {
                result_flag = ctx.caller == &ctx.session.owner;
            }
            
            GuardOp::CheckExpiry { timestamp } => {
                result_flag = ctx.clock.unix_timestamp < *timestamp;
            }
            
            GuardOp::CheckNotBefore { timestamp } => {
                result_flag = ctx.clock.unix_timestamp >= *timestamp;
            }
            
            GuardOp::CheckUsageLimit { limit } => {
                result_flag = ctx.session.usage_count < *limit;
            }
            
            // ===== Control Flow Opcodes =====
            GuardOp::JumpIfFalse { offset } => {
                if !result_flag {
                    instruction_pointer += *offset as usize;
                    continue; // Skip the normal increment
                }
            }
            
            GuardOp::Terminate => {
                return Ok(true);
            }
            
            GuardOp::Abort => {
                return Ok(false);
            }
            
            // ===== External Interaction Opcode =====
            GuardOp::Invoke { manifest_index } => {
                let entry = guard.cpi_manifest
                    .get(*manifest_index as usize)
                    .ok_or(KernelError::InvalidParameters)?;
                
                // Build accounts for CPI
                let mut cpi_accounts = Vec::new();
                for &idx in &entry.account_indices {
                    let account = ctx.remaining_accounts
                        .get(idx as usize)
                        .ok_or(KernelError::InvalidParameters)?;
                    cpi_accounts.push(account.clone());
                }
                
                // Build and invoke instruction
                let ix = solana_program::instruction::Instruction {
                    program_id: entry.program_id,
                    accounts: cpi_accounts.iter().map(|a| AccountMeta {
                        pubkey: *a.key,
                        is_signer: a.is_signer,
                        is_writable: a.is_writable,
                    }).collect(),
                    data: entry.data.clone(),
                };
                
                solana_program::program::invoke(&ix, &cpi_accounts)?;
                
                // Get return data
                let (returned_program, return_data) = solana_program::program::get_return_data()
                    .ok_or(KernelError::ExternalGuardNoReturnData)?;
                
                require!(
                    returned_program == entry.program_id,
                    KernelError::ExternalGuardInvalidReturn
                );
                
                // Parse return as bool
                result_flag = return_data.first()
                    .map(|&b| b != 0)
                    .unwrap_or(false);
            }
        }
        
        instruction_pointer += 1;
    }
    
    // Execution fell off the end - implicit failure
    Ok(false)
}

