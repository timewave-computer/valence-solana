//! Kernel-specific instruction builders

use crate::Result;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

// Import kernel types
use valence_kernel::{
    state::CreateSessionParams,
    KernelOperation, OperationBatch,
    PROGRAM_ID as KERNEL_PROGRAM_ID,
};

/// Create a transfer instruction
pub fn transfer_instruction(from: Pubkey, to: Pubkey, lamports: u64) -> Instruction {
    solana_sdk::system_instruction::transfer(&from, &to, lamports)
}

/// Create account initialization instruction
pub fn create_account_instruction(
    from: Pubkey,
    to: Pubkey,
    lamports: u64,
    space: u64,
    owner: &Pubkey,
) -> Instruction {
    solana_sdk::system_instruction::create_account(&from, &to, lamports, space, owner)
}

/// Build valence-kernel session creation instruction
pub fn create_session_instruction(
    payer: Pubkey,
    session: Pubkey,
    shard: Pubkey,
    _params: CreateSessionParams,
) -> Result<Instruction> {
    // Note: This is a simplified version. Real implementation would use
    // Anchor's instruction builders or manually construct the instruction data.
    
    let accounts = vec![
        AccountMeta::new(payer, true),        // payer
        AccountMeta::new(session, false),     // session account
        AccountMeta::new_readonly(shard, false), // shard account
        AccountMeta::new_readonly(solana_sdk::system_program::ID, false), // system program
        AccountMeta::new_readonly(solana_sdk::sysvar::clock::ID, false),   // clock sysvar
    ];

    // Serialize parameters (simplified - real implementation would use Anchor borsh)
    let data = vec![
        // Instruction discriminator for create_session_account
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        // Parameters would be serialized here
    ];

    Ok(Instruction {
        program_id: KERNEL_PROGRAM_ID,
        accounts,
        data,
    })
}

/// Build batch execution instruction
pub fn execute_batch_instruction(
    session: Pubkey,
    alt: Option<Pubkey>,
    batch: OperationBatch,
) -> Result<Instruction> {
    let mut accounts = vec![
        AccountMeta::new(session, false),     // session account
        AccountMeta::new_readonly(solana_sdk::sysvar::clock::ID, false), // clock sysvar
    ];

    // Add ALT if present
    if let Some(alt_pubkey) = alt {
        accounts.push(AccountMeta::new(alt_pubkey, false));
    }

    // Add accounts from batch
    for account_pubkey in &batch.accounts {
        accounts.push(AccountMeta::new(*account_pubkey, false));
    }

    // Serialize batch data (simplified)
    let mut data = vec![
        // Instruction discriminator for execute_batch
        0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80,
    ];
    
    // Serialize operations count
    data.extend_from_slice(&(batch.operations.len() as u32).to_le_bytes());
    
    // Serialize each operation (simplified)
    for op in batch.operations.iter().flatten() {
        match op {
            KernelOperation::BorrowAccount { account_index, mode } => {
                data.push(0x01); // Operation type
                data.push(*account_index);
                data.push(*mode);
            }
            KernelOperation::ReleaseAccount { account_index } => {
                data.push(0x02); // Operation type
                data.push(*account_index);
            }
            // Add other operation types as needed
            _ => {} // Handle other variants if needed
        }
    }

    Ok(Instruction {
        program_id: KERNEL_PROGRAM_ID,
        accounts,
        data,
    })
}

/// Build guard account creation instruction
pub fn create_guard_instruction(
    payer: Pubkey,
    guard: Pubkey,
    session: Pubkey,
    _allow_unregistered_cpi: bool,
) -> Result<Instruction> {
    let accounts = vec![
        AccountMeta::new(payer, true),
        AccountMeta::new(guard, false),
        AccountMeta::new_readonly(session, false),
        AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
    ];

    let data = vec![
        // Instruction discriminator for create_guard_account
        0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0, 0x01, 0x02,
        // Session pubkey (32 bytes)
    ];

    Ok(Instruction {
        program_id: KERNEL_PROGRAM_ID,
        accounts,
        data,
    })
}

/// Build child account creation instruction
pub fn create_child_account_instruction(
    payer: Pubkey,
    session: Pubkey,
    child_account: Pubkey,
    namespace_suffix: String,
    initial_lamports: u64,
    space: u64,
    owner_program: Pubkey,
) -> Result<Instruction> {
    let accounts = vec![
        AccountMeta::new(session, false),
        AccountMeta::new(child_account, false),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
        AccountMeta::new_readonly(solana_sdk::sysvar::rent::ID, false),
    ];

    let mut data = vec![
        // Instruction discriminator for create_child_account
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
    ];
    
    // Serialize parameters
    data.extend_from_slice(&(namespace_suffix.len() as u32).to_le_bytes());
    data.extend_from_slice(namespace_suffix.as_bytes());
    data.extend_from_slice(&initial_lamports.to_le_bytes());
    data.extend_from_slice(&space.to_le_bytes());
    data.extend_from_slice(&owner_program.to_bytes());

    Ok(Instruction {
        program_id: KERNEL_PROGRAM_ID,
        accounts,
        data,
    })
}

/// Build account lookup table (ALT) management instruction
pub fn manage_alt_instruction(
    authority: Pubkey,
    session: Pubkey,
    alt: Pubkey,
    add_accounts: Vec<Pubkey>,
    remove_accounts: Vec<Pubkey>,
) -> Result<Instruction> {
    let mut accounts = vec![
        AccountMeta::new(authority, true),
        AccountMeta::new(session, false),
        AccountMeta::new(alt, false),
        AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
    ];

    // Add accounts to be managed
    for account in &add_accounts {
        accounts.push(AccountMeta::new_readonly(*account, false));
    }

    let mut data = vec![
        // Instruction discriminator for manage_alt
        0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00,
    ];
    
    // Serialize add accounts
    data.extend_from_slice(&(add_accounts.len() as u32).to_le_bytes());
    for account in &add_accounts {
        data.extend_from_slice(&account.to_bytes());
    }
    
    // Serialize remove accounts
    data.extend_from_slice(&(remove_accounts.len() as u32).to_le_bytes());
    for account in &remove_accounts {
        data.extend_from_slice(&account.to_bytes());
    }

    Ok(Instruction {
        program_id: KERNEL_PROGRAM_ID,
        accounts,
        data,
    })
}

/// Estimate compute units for kernel operations
pub fn estimate_kernel_operation_compute(operation: &KernelOperation) -> u64 {
    match operation {
        KernelOperation::BorrowAccount { .. } => 5_000,
        KernelOperation::ReleaseAccount { .. } => 3_000,
        KernelOperation::CallRegisteredFunction { .. } => 10_000,
        KernelOperation::UnsafeRawCpi { .. } => 15_000,
    }
}