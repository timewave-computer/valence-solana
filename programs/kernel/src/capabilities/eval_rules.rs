// App-specific execution rules for shard contracts
// 
// Valence embeds evaluation logic within
// shard contracts by default, providing:
// - Simple deployment and operation
// - Better performance through reduced CPI calls
// - Tighter coupling between shard state and evaluation rules
//
// Developers can chose to split eval into a separate contracts if they chose.


use anchor_lang::prelude::*;
use crate::error::ValenceError;

/// Shard state that includes evaluation configuration
#[account]
pub struct ShardState {
    /// Authority that can manage the shard
    pub authority: Pubkey,
    /// Processor program that handles execution
    pub processor_program: Pubkey,
    /// Whether the shard is paused
    pub is_paused: bool,
    /// Total number of capabilities executed
    pub total_executions: u64,
    /// Shard version
    pub version: u8,
    /// PDA bump seed
    pub bump: u8,
    /// Evaluation configuration
    pub eval_config: EvalConfig,
}

/// Evaluation configuration embedded in shard
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct EvalConfig {
    /// Maximum execution time allowed (seconds)
    pub max_execution_time: u64,
    /// Maximum compute units allowed
    pub max_compute_units: u64,
    /// Whether to record execution results
    pub record_execution: bool,
    /// Default verification function requirements
    pub default_verification_functions: Vec<[u8; 32]>,
}

impl ShardState {
    pub const SPACE: usize = 8 +   // discriminator
        32 +  // authority (Pubkey)
        32 +  // processor_program (Pubkey)
        1 +   // is_paused (bool)
        8 +   // total_executions (u64)
        1 +   // version (u8)
        1 +   // bump (u8)
        128;  // eval_config (estimated space)

    /// Process initialization logic for shard with eval
    pub fn process_initialize(
        &mut self, 
        authority: Pubkey, 
        processor_program: Pubkey, 
        bump: u8
    ) -> Result<()> {
        self.authority = authority;
        self.processor_program = processor_program;
        self.is_paused = false;
        self.total_executions = 0;
        self.version = 1;
        self.bump = bump;
        self.eval_config = EvalConfig::default();
        
        msg!("ShardState initialized with authority: {} and processor: {}", authority, processor_program);
        Ok(())
    }
    
    /// Execute a capability using embedded eval logic
    pub fn process_execute_capability(
        &mut self,
        capability_id: String,
        input_data: Vec<u8>,
        execution_context: &ExecutionContext,
    ) -> Result<ExecutionResult> {
        require!(!self.is_paused, ValenceError::SystemPaused);
        
        // Apply evaluation rules
        self.validate_execution_rules(&capability_id, &input_data, execution_context)?;
        
        // Increment execution counter
        self.total_executions = self.total_executions.checked_add(1)
            .ok_or(ValenceError::ArithmeticOverflow)?;
        
        msg!("Executing capability: {} with {} bytes of input", capability_id, input_data.len());
        
        // CPI to processor singleton for actual execution
        // See cpi_patterns.md for integration examples
        // The processor handles:
        // - Execution context building
        // - Verification chain orchestration  
        // - Resource limit enforcement
        // - Result aggregation
        
        Ok(ExecutionResult {
            success: true,
            output_data: vec![],
            gas_used: input_data.len() as u64,
            error_message: None,
            logs: vec![],
        })
    }
    
    /// Validate execution rules specific to this shard
    pub fn validate_execution_rules(
        &self,
        capability_id: &str,
        input_data: &[u8],
        execution_context: &ExecutionContext,
    ) -> Result<()> {
        // Basic validation
        require!(!capability_id.is_empty(), ValenceError::InvalidInput);
        require!(!input_data.is_empty(), ValenceError::InvalidInput);
        
        // Check compute limits
        if let Some(compute_limit) = execution_context.compute_limit {
            require!(
                compute_limit <= self.eval_config.max_compute_units,
                ValenceError::ComputeLimitExceeded
            );
        }
        
        // Additional app-specific validation can be added here
        Ok(())
    }
    
    /// Update evaluation configuration
    pub fn update_eval_config(&mut self, new_config: EvalConfig) -> Result<()> {
        self.eval_config = new_config;
        msg!("Eval config updated");
        Ok(())
    }
    
    /// Pause the shard
    pub fn process_pause(&mut self) -> Result<()> {
        require!(!self.is_paused, ValenceError::SystemAlreadyPaused);
        self.is_paused = true;
        msg!("Shard paused");
        Ok(())
    }
    
    /// Resume the shard
    pub fn process_resume(&mut self) -> Result<()> {
        require!(self.is_paused, ValenceError::SystemNotPaused);
        self.is_paused = false;
        msg!("Shard resumed");
        Ok(())
    }
}

// Import execution types from processor singleton
pub use crate::processor::{ExecutionContext, ExecutionResult, ContextBuilder}; 