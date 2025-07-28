use crate::{Result, SessionHandle};
use anchor_lang::prelude::*;
use solana_sdk::instruction::Instruction;
use valence_kernel::{OperationBatch, SessionOperation, ProgramManifestEntry};

/// Atomic operation types
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub enum AtomicOperation {
    /// Transfer tokens
    Transfer {
        from: Pubkey,
        to: Pubkey,
        amount: u64,
    },
    /// Mint tokens
    Mint {
        mint: Pubkey,
        to: Pubkey,
        amount: u64,
    },
    /// Burn tokens
    Burn {
        mint: Pubkey,
        from: Pubkey,
        amount: u64,
    },
    /// Custom operation
    Custom {
        discriminator: [u8; 8],
        data: Vec<u8>,
    },
}

/// Builder for atomic operations
pub struct AtomicBuilder<'a> {
    #[allow(dead_code)] // Will be used in future implementation
    session: &'a SessionHandle<'a>,
    operations: Vec<AtomicOperation>,
}

impl<'a> AtomicBuilder<'a> {
    /// Create a new atomic builder
    pub fn new(session: &'a SessionHandle<'a>) -> Self {
        Self {
            session,
            operations: Vec::new(),
        }
    }

    /// Add a transfer operation
    pub fn transfer(mut self, from: Pubkey, to: Pubkey, amount: u64) -> Self {
        self.operations.push(AtomicOperation::Transfer {
            from,
            to,
            amount,
        });
        self
    }

    /// Add a mint operation
    pub fn mint(mut self, mint: Pubkey, to: Pubkey, amount: u64) -> Self {
        self.operations.push(AtomicOperation::Mint { mint, to, amount });
        self
    }

    /// Add a burn operation
    pub fn burn(mut self, mint: Pubkey, from: Pubkey, amount: u64) -> Self {
        self.operations.push(AtomicOperation::Burn { mint, from, amount });
        self
    }

    /// Add a custom operation
    pub fn custom(mut self, discriminator: [u8; 8], data: Vec<u8>) -> Self {
        self.operations.push(AtomicOperation::Custom {
            discriminator,
            data,
        });
        self
    }

    /// Build an OperationBatch from atomic operations
    pub fn build_batch(self) -> Result<OperationBatch> {
        // Convert AtomicOperations to SessionOperations
        let mut session_operations = Vec::new();
        let mut program_manifest = Vec::new();
        
        for op in self.operations {
            match op {
                AtomicOperation::Transfer { from: _, to: _, amount: _ } => {
                    // This would require a token program CPI
                    // For now, convert to a custom operation
                    session_operations.push(SessionOperation::Custom {
                        program_id: spl_token::id(),
                        discriminator: [0, 1, 0, 0, 0, 0, 0, 0], // Transfer discriminator
                        data: vec![], // Would contain transfer instruction data
                    });
                    
                    // Add token program to manifest if not present
                    if !program_manifest.iter().any(|p: &ProgramManifestEntry| p.program_id == spl_token::id()) {
                        program_manifest.push(ProgramManifestEntry {
                            program_id: spl_token::id(),
                        });
                    }
                }
                AtomicOperation::Mint { mint: _, to: _, amount: _ } => {
                    session_operations.push(SessionOperation::Custom {
                        program_id: spl_token::id(),
                        discriminator: [0, 2, 0, 0, 0, 0, 0, 0], // MintTo discriminator
                        data: vec![], // Would contain mint instruction data
                    });
                    
                    if !program_manifest.iter().any(|p: &ProgramManifestEntry| p.program_id == spl_token::id()) {
                        program_manifest.push(ProgramManifestEntry {
                            program_id: spl_token::id(),
                        });
                    }
                }
                AtomicOperation::Burn { mint: _, from: _, amount: _ } => {
                    session_operations.push(SessionOperation::Custom {
                        program_id: spl_token::id(),
                        discriminator: [0, 3, 0, 0, 0, 0, 0, 0], // Burn discriminator
                        data: vec![], // Would contain burn instruction data
                    });
                    
                    if !program_manifest.iter().any(|p: &ProgramManifestEntry| p.program_id == spl_token::id()) {
                        program_manifest.push(ProgramManifestEntry {
                            program_id: spl_token::id(),
                        });
                    }
                }
                AtomicOperation::Custom { discriminator, data } => {
                    // Use generic custom operation - program would need to be specified separately
                    session_operations.push(SessionOperation::Custom {
                        program_id: Pubkey::default(), // This would need to be specified
                        discriminator,
                        data,
                    });
                }
            }
        }
        
        Ok(OperationBatch {
            operations: session_operations,
            auto_release: true,
            program_manifest,
        })
    }
    
    /// Build instruction for atomic operations (deprecated)
    #[deprecated(note = "Use build_batch() and execute_session_operations_instruction instead")]
    pub fn build_instruction(&self) -> Result<Instruction> {
        // This would need to be implemented differently as build_batch consumes self
        unimplemented!("Use build_batch() separately and then execute_session_operations_instruction")
    }

    /// Get the number of operations
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

/// Helper macro for creating atomic batches
#[macro_export]
macro_rules! atomic_batch {
    ($session:expr, $($op:expr),+ $(,)?) => {{
        let mut builder = $crate::AtomicBuilder::new($session);
        $(
            builder = $op(builder);
        )+
        builder
    }};
}