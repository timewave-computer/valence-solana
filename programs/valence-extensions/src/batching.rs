//! Minimal batching support

use anchor_lang::prelude::*;

/// Batch multiple operations
#[derive(Default)]
pub struct Batch<'a> {
    operations: Vec<Box<dyn FnOnce() -> Result<()> + 'a>>,
}

impl<'a> Batch<'a> {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn add_operation<F>(mut self, op: F) -> Self 
    where 
        F: FnOnce() -> Result<()> + 'a
    {
        self.operations.push(Box::new(op));
        self
    }
    
    pub fn execute(self) -> Result<()> {
        for op in self.operations {
            op()?;
        }
        Ok(())
    }
    
    pub fn execute_atomic(self) -> Result<()> {
        // Execute all operations, return on first error
        let total_operations = self.operations.len();
        
        for (i, op) in self.operations.into_iter().enumerate() {
            if let Err(e) = op() {
                // On first error, return with operation index
                msg!("Batch operation {} failed: {:?}", i, e);
                return Err(ErrorCode::BatchOperationFailed.into());
            }
        }
        
        // All operations succeeded
        msg!("Batch executed successfully: {} operations", total_operations);
        Ok(())
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Batch operation failed")]
    BatchOperationFailed,
}