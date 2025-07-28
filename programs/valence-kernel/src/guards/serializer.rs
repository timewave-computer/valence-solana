// Guard serializer module
// Serializes Guard enums into flattened opcodes for efficient evaluation
use anchor_lang::prelude::*;
use super::{Guard, GuardOp, SerializedGuard, CpiCallEntry};

/// Compile a Guard into a SerializedGuard (flattened opcodes)
pub fn compile_high_level_guard(guard: &Guard) -> Result<SerializedGuard> {
    let mut opcodes = Vec::new();
    let mut cpi_manifest = Vec::new();
    
    // Flatten the guard into opcodes
    serialize_guard_tree(guard, &mut opcodes, &mut cpi_manifest)?;
    
    // Add termination
    opcodes.push(GuardOp::Terminate);
    
    SerializedGuard::new_with_manifest(opcodes, cpi_manifest)
}

/// Recursively serialize a guard into opcodes
fn serialize_guard_tree(
    guard: &Guard, 
    opcodes: &mut Vec<GuardOp>, 
    cpi_manifest: &mut Vec<CpiCallEntry>
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
            msg!("Warning: Permission guards not fully implemented in serializer");
            opcodes.push(GuardOp::Abort);
        }
        Guard::PermissionAccount { account: _, required_level: _ } => {
            // Would need CPI to check permission account
            msg!("Warning: PermissionAccount guards not fully implemented in serializer");
            opcodes.push(GuardOp::Abort);
        }
        Guard::UsageLimit { max } => {
            opcodes.push(GuardOp::CheckUsageLimit { limit: *max });
            opcodes.push(GuardOp::JumpIfFalse { offset: 1 });
        }
        Guard::And(left, right) => {
            serialize_guard_tree(left, opcodes, cpi_manifest)?;
            let jump_to_end = opcodes.len();
            opcodes.push(GuardOp::JumpIfFalse { offset: 0 }); // Will be patched
            
            serialize_guard_tree(right, opcodes, cpi_manifest)?;
            
            // Patch the jump offset
            let end_offset = opcodes.len() - jump_to_end - 1;
            if end_offset > u8::MAX as usize {
                return Err(crate::errors::KernelError::GuardDataTooLarge.into());
            }
            opcodes[jump_to_end] = GuardOp::JumpIfFalse { offset: end_offset as u8 };
        }
        Guard::Or(left, right) => {
            // For OR: if left succeeds, skip right
            serialize_guard_tree(left, opcodes, cpi_manifest)?;
            let jump_past_right = opcodes.len();
            opcodes.push(GuardOp::JumpIfFalse { offset: 0 }); // Will be patched
            
            // Left succeeded, terminate with success
            opcodes.push(GuardOp::Terminate);
            
            // Patch jump to here (right evaluation)
            let right_start = opcodes.len() - jump_past_right - 1;
            if right_start > u8::MAX as usize {
                return Err(crate::errors::KernelError::GuardDataTooLarge.into());
            }
            opcodes[jump_past_right] = GuardOp::JumpIfFalse { offset: right_start as u8 };
            
            serialize_guard_tree(right, opcodes, cpi_manifest)?;
        }
        Guard::Not(inner) => {
            serialize_guard_tree(inner, opcodes, cpi_manifest)?;
            // Invert the result by jumping if true instead of false
            // This is complex - for now, treat as unimplemented
            msg!("Warning: Not guards not fully implemented in serializer");
            opcodes.push(GuardOp::Abort);
        }
        Guard::External { program, data, required_accounts } => {
            // Add to CPI manifest
            let manifest_index = cpi_manifest.len();
            if manifest_index > u8::MAX as usize {
                return Err(crate::errors::KernelError::GuardDataTooLarge.into());
            }
            
            cpi_manifest.push(CpiCallEntry {
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