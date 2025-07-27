// Guard compiler module
// Compiles Guard enums into flattened opcodes for efficient evaluation
use anchor_lang::prelude::*;
use super::{Guard, GuardOp, CompiledGuard, CPIManifestEntry};

/// Compile a Guard into a CompiledGuard (flattened opcodes)
pub fn compile_guard(guard: &Guard) -> Result<CompiledGuard> {
    let mut opcodes = Vec::new();
    let mut cpi_manifest = Vec::new();
    
    // Flatten the guard into opcodes
    compile_guard_recursive(guard, &mut opcodes, &mut cpi_manifest)?;
    
    // Add termination
    opcodes.push(GuardOp::Terminate);
    
    CompiledGuard::new_with_manifest(opcodes, cpi_manifest)
}

/// Recursively compile a guard into opcodes
fn compile_guard_recursive(
    guard: &Guard, 
    opcodes: &mut Vec<GuardOp>, 
    cpi_manifest: &mut Vec<CPIManifestEntry>
) -> Result<()> {
    match guard {
        Guard::AlwaysTrue => {
            // Always terminates with success - no-op, will hit Terminate
        }
        Guard::AlwaysFalse => {
            opcodes.push(GuardOp::Abort);
        }
        Guard::OwnerOnly => {
            opcodes.push(GuardOp::CheckOwner);
            opcodes.push(GuardOp::JumpIfFalse { offset: 1 });
        }
        Guard::Expiration { expires_at } => {
            opcodes.push(GuardOp::CheckExpiry { timestamp: *expires_at });
            opcodes.push(GuardOp::JumpIfFalse { offset: 1 });
        }
        Guard::Permission { required: _ } => {
            // For now, treat as always false - would need permission account check
            msg!("Warning: Permission guards not fully implemented in compiler");
            opcodes.push(GuardOp::Abort);
        }
        Guard::PermissionAccount { account: _, required_level: _ } => {
            // Would need CPI to check permission account
            msg!("Warning: PermissionAccount guards not fully implemented in compiler");
            opcodes.push(GuardOp::Abort);
        }
        Guard::UsageLimit { max } => {
            opcodes.push(GuardOp::CheckUsageLimit { limit: *max });
            opcodes.push(GuardOp::JumpIfFalse { offset: 1 });
        }
        Guard::And(left, right) => {
            compile_guard_recursive(left, opcodes, cpi_manifest)?;
            let jump_to_end = opcodes.len();
            opcodes.push(GuardOp::JumpIfFalse { offset: 0 }); // Will be patched
            
            compile_guard_recursive(right, opcodes, cpi_manifest)?;
            
            // Patch the jump offset
            let end_offset = opcodes.len() - jump_to_end - 1;
            if end_offset > u8::MAX as usize {
                return Err(crate::errors::ValenceError::GuardDataTooLarge.into());
            }
            opcodes[jump_to_end] = GuardOp::JumpIfFalse { offset: end_offset as u8 };
        }
        Guard::Or(left, right) => {
            // For OR: if left succeeds, skip right
            compile_guard_recursive(left, opcodes, cpi_manifest)?;
            let jump_past_right = opcodes.len();
            opcodes.push(GuardOp::JumpIfFalse { offset: 0 }); // Will be patched
            
            // Left succeeded, terminate with success
            opcodes.push(GuardOp::Terminate);
            
            // Patch jump to here (right evaluation)
            let right_start = opcodes.len() - jump_past_right - 1;
            if right_start > u8::MAX as usize {
                return Err(crate::errors::ValenceError::GuardDataTooLarge.into());
            }
            opcodes[jump_past_right] = GuardOp::JumpIfFalse { offset: right_start as u8 };
            
            compile_guard_recursive(right, opcodes, cpi_manifest)?;
        }
        Guard::Not(inner) => {
            compile_guard_recursive(inner, opcodes, cpi_manifest)?;
            // Invert the result by jumping if true instead of false
            // This is complex - for now, treat as unimplemented
            msg!("Warning: Not guards not fully implemented in compiler");
            opcodes.push(GuardOp::Abort);
        }
        Guard::External { program, data, required_accounts } => {
            // Add to CPI manifest
            let manifest_index = cpi_manifest.len();
            if manifest_index > u8::MAX as usize {
                return Err(crate::errors::ValenceError::GuardDataTooLarge.into());
            }
            
            cpi_manifest.push(CPIManifestEntry {
                program_id: *program,
                account_indices: (0..required_accounts.len().min(8)).map(|i| i as u8).collect(),
                data: data.clone(),
            });
            
            opcodes.push(GuardOp::Invoke { manifest_index: manifest_index as u8 });
            opcodes.push(GuardOp::JumpIfFalse { offset: 1 });
        }
    }
    
    Ok(())
}