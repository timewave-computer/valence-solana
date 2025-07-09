// Diff instruction handlers and types

use anchor_lang::prelude::*;
use crate::diff::DiffState;

// ======================= TYPES =======================

/// Represents a unified diff operation combining both positional and key-value operations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum DiffOperation {
    // Key-value operations
    /// Add a new key-value pair
    Add { key: String, value: Vec<u8> },
    /// Update an existing key-value pair
    Update { key: String, old_value: Vec<u8>, new_value: Vec<u8> },
    /// Remove a key-value pair
    Remove { key: String, value: Vec<u8> },
    /// Move/rename a key
    Move { old_key: String, new_key: String, value: Vec<u8> },
    
    // Positional operations
    /// Insert data at a specific position
    Insert { position: u64, data: Vec<u8> },
    /// Delete data at a specific position
    Delete { position: u64, length: u64 },
    /// Replace data at a specific position
    Replace { position: u64, data: Vec<u8> },
}

/// A batch of diff operations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DiffBatch {
    /// Unique identifier for this batch
    pub batch_id: [u8; 32],
    /// Operations in this batch
    pub operations: Vec<DiffOperation>,
    /// Timestamp when batch was created
    pub created_at: i64,
    /// Source of the diff
    pub source: Pubkey,
    /// Target for the diff
    pub target: Pubkey,
}

/// Result of diff processing
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DiffResult {
    /// Whether diff was applied successfully
    pub success: bool,
    /// Number of operations processed
    pub operations_processed: u32,
    /// Number of operations failed
    pub operations_failed: u32,
    /// Gas consumed
    pub gas_used: u64,
    /// Error message if failed
    pub error: Option<String>,
}

/// Diff validation result
#[derive(Debug)]
pub struct DiffValidation {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl DiffOperation {
    /// Calculate the size of this operation
    pub fn size(&self) -> usize {
        match self {
            DiffOperation::Add { key, value } => key.len() + value.len(),
            DiffOperation::Update { key, old_value, new_value } => {
                key.len() + old_value.len() + new_value.len()
            }
            DiffOperation::Remove { key, value } => key.len() + value.len(),
            DiffOperation::Move { old_key, new_key, value } => {
                old_key.len() + new_key.len() + value.len()
            }
            DiffOperation::Insert { data, .. } => data.len(),
            DiffOperation::Delete { length, .. } => *length as usize,
            DiffOperation::Replace { data, .. } => data.len(),
        }
    }

    /// Check if this operation conflicts with another
    pub fn conflicts_with(&self, other: &DiffOperation) -> bool {
        match (self, other) {
            // Key-value conflicts
            (DiffOperation::Add { key: k1, .. }, DiffOperation::Add { key: k2, .. }) => k1 == k2,
            (DiffOperation::Update { key: k1, .. }, DiffOperation::Update { key: k2, .. }) => k1 == k2,
            (DiffOperation::Remove { key: k1, .. }, DiffOperation::Update { key: k2, .. }) => k1 == k2,
            (DiffOperation::Update { key: k1, .. }, DiffOperation::Remove { key: k2, .. }) => k1 == k2,
            (DiffOperation::Move { new_key: k1, .. }, DiffOperation::Add { key: k2, .. }) => k1 == k2,
            
            // Positional conflicts
            (DiffOperation::Insert { position: p1, .. }, DiffOperation::Insert { position: p2, .. }) => p1 == p2,
            (DiffOperation::Delete { position: p1, .. }, DiffOperation::Delete { position: p2, .. }) => p1 == p2,
            (DiffOperation::Replace { position: p1, .. }, DiffOperation::Replace { position: p2, .. }) => p1 == p2,
            
            _ => false,
        }
    }
}

impl DiffBatch {
    /// Create a new diff batch
    pub fn new(source: Pubkey, target: Pubkey) -> Self {
        Self {
            batch_id: [0; 32], // Should be generated properly
            operations: Vec::new(),
            created_at: Clock::get().map(|c| c.unix_timestamp).unwrap_or(0),
            source,
            target,
        }
    }

    /// Add an operation to the batch
    pub fn add_operation(&mut self, operation: DiffOperation) {
        self.operations.push(operation);
    }

    /// Get total size of the batch
    pub fn total_size(&self) -> usize {
        self.operations.iter().map(|op| op.size()).sum()
    }

    /// Validate the batch
    pub fn validate(&self) -> DiffValidation {
        let mut validation = DiffValidation {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Check for empty batch
        if self.operations.is_empty() {
            validation.warnings.push("Batch contains no operations".to_string());
        }

        // Check for conflicts within the batch
        for i in 0..self.operations.len() {
            for j in (i + 1)..self.operations.len() {
                if self.operations[i].conflicts_with(&self.operations[j]) {
                    validation.is_valid = false;
                    validation.errors.push(format!(
                        "Operation {} conflicts with operation {}", i, j
                    ));
                }
            }
        }

        validation
    }
}

// ======================= INSTRUCTION HANDLERS =======================

/// Initialize the diff singleton
pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let diff_state = &mut ctx.accounts.diff_state;
    diff_state.authority = ctx.accounts.authority.key();
    diff_state.total_diffs = 0;
    diff_state.bump = ctx.bumps.diff_state;
    
    msg!("Diff singleton initialized");
    Ok(())
}

/// Calculate diff between two states
pub fn calculate_diff(
    ctx: Context<CalculateDiff>,
    state_a: Vec<u8>,
    state_b: Vec<u8>,
) -> Result<()> {
    let diff_state = &mut ctx.accounts.diff_state;
    
    // Basic diff calculation (to be enhanced)
    diff_state.total_diffs += 1;
    
    msg!("Calculated diff between states of {} and {} bytes", state_a.len(), state_b.len());
    Ok(())
}

/// Process diffs atomically
pub fn process_diffs(
    _ctx: Context<ProcessDiffs>,
    diffs: Vec<DiffOperation>,
) -> Result<()> {
    // TODO: Implement atomic diff processing
    msg!("Processing {} diff operations", diffs.len());
    Ok(())
}

/// Verify diff integrity
pub fn verify_diff_integrity(
    _ctx: Context<VerifyDiffIntegrity>,
    diff_hash: [u8; 32],
) -> Result<()> {
    // TODO: Implement diff integrity verification
    msg!("Verifying diff integrity for hash: {:?}", diff_hash);
    Ok(())
}

// ======================= ACCOUNT CONTEXTS =======================

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = DiffState::SPACE,
        seeds = [b"diff_state"],
        bump
    )]
    pub diff_state: Account<'info, DiffState>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CalculateDiff<'info> {
    #[account(mut)]
    pub diff_state: Account<'info, DiffState>,
    
    pub caller: Signer<'info>,
}

#[derive(Accounts)]
pub struct ProcessDiffs<'info> {
    #[account(mut)]
    pub diff_state: Account<'info, DiffState>,
    
    pub caller: Signer<'info>,
}

#[derive(Accounts)]
pub struct VerifyDiffIntegrity<'info> {
    #[account(mut)]
    pub diff_state: Account<'info, DiffState>,
    
    pub caller: Signer<'info>,
} 