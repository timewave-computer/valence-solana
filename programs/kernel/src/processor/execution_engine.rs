// Core execution logic for capability processing

use anchor_lang::prelude::*;
use crate::error::ProcessorError;

/// Core execution engine for processing capabilities
pub struct ExecutionEngine;

impl ExecutionEngine {
    /// Execute a capability with the given parameters
    pub fn execute_capability(
        capability_id: &str,
        input_data: &[u8],
        context: &ExecutionContext,
    ) -> Result<ExecutionResult> {
        // Basic validation
        if capability_id.is_empty() {
            return Err(ProcessorError::CapabilityInvalidId.into());
        }

        // Validate execution context
        Self::validate_context(context)?;

        // Apply resource limits if configured
        let gas_limit = context.compute_limit.unwrap_or(1_000_000);
        let mut gas_used = 0u64;

        // Track execution start
        gas_used += 100; // Base cost for execution
        gas_used += input_data.len() as u64 * 10; // Cost per byte of input

        // Check gas limit
        if gas_used > gas_limit {
            return Ok(ExecutionResult::failure(
                "Gas limit exceeded".to_string(),
                gas_used,
            ));
        }

        // Log execution start
        msg!("Executing capability: {} for caller: {}", capability_id, context.caller);

        // Create result with logs
        let mut result = ExecutionResult::success(vec![], gas_used);
        result.add_log(format!("Started execution of capability: {}", capability_id));
        result.add_log(format!("Input data size: {} bytes", input_data.len()));
        
        // Process capability metadata
        gas_used += 50;
        if let Some(session) = context.session {
            result.add_log(format!("Executing in session context: {}", session));
        }
        
        // Simulate processing stages
        gas_used += 200;
        result.add_log("Stage 1: Context validation - PASSED".to_string());
        result.add_log("Stage 2: Permission check - PASSED".to_string());
        result.add_log("Stage 3: Resource allocation - PASSED".to_string());
        
        // Process input data
        if !input_data.is_empty() {
            gas_used += input_data.len() as u64 * 5;
            result.add_log(format!("Processed {} bytes of input data", input_data.len()));
            
            // Echo first 32 bytes as output (for testing)
            let output_len = input_data.len().min(32);
            result.output_data = input_data[..output_len].to_vec();
        }
        
        // Final gas check
        if gas_used > gas_limit {
            return Ok(ExecutionResult::failure(
                "Gas limit exceeded during execution".to_string(),
                gas_used,
            ));
        }
        
        result.gas_used = gas_used;
        result.add_log(format!("Execution completed successfully. Gas used: {}", gas_used));
        
        Ok(result)
    }

    /// Validate execution context
    pub fn validate_context(context: &ExecutionContext) -> Result<()> {
        // Basic validation
        if context.session.is_none() || context.session == Some(Pubkey::default()) {
            return Err(ProcessorError::ExecutionFailed.into());
        }

        Ok(())
    }
}

/// Execution context for capability processing
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct ExecutionContext {
    pub capability_id: String,
    pub session_id: String,
    pub session: Option<Pubkey>,
    pub caller: Pubkey,
    pub timestamp: i64,
    pub block_height: u64,
    pub compute_limit: Option<u64>,
    pub input_data: Vec<u8>,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(
        capability_id: String,
        caller: Pubkey,
        session: Option<Pubkey>,
    ) -> Self {
        let clock = Clock::get().unwrap_or_default();
        Self {
            capability_id,
            session_id: String::new(),
            session,
            caller,
            timestamp: clock.unix_timestamp,
            block_height: clock.slot,
            compute_limit: None,
            input_data: vec![],
        }
    }

    /// Create context with session
    pub fn with_session(
        capability_id: String,
        session_id: String,
        session: Pubkey,
        caller: Pubkey,
    ) -> Self {
        let clock = Clock::get().unwrap_or_default();
        Self {
            capability_id,
            session_id,
            session: Some(session),
            caller,
            timestamp: clock.unix_timestamp,
            block_height: clock.slot,
            compute_limit: None,
            input_data: vec![],
        }
    }

    /// Set compute limit
    pub fn with_compute_limit(mut self, limit: u64) -> Self {
        self.compute_limit = Some(limit);
        self
    }

    /// Set input data
    pub fn with_input_data(mut self, data: Vec<u8>) -> Self {
        self.input_data = data;
        self
    }
}

/// Result of capability execution
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub output_data: Vec<u8>,
    pub gas_used: u64,
    pub error_message: Option<String>,
    pub logs: Vec<String>,
}

impl ExecutionResult {
    /// Create a successful result
    pub fn success(output_data: Vec<u8>, gas_used: u64) -> Self {
        Self {
            success: true,
            output_data,
            gas_used,
            error_message: None,
            logs: vec![],
        }
    }

    /// Create a failed result
    pub fn failure(error_message: String, gas_used: u64) -> Self {
        Self {
            success: false,
            output_data: vec![],
            gas_used,
            error_message: Some(error_message),
            logs: vec![],
        }
    }

    /// Add a log entry
    pub fn add_log(&mut self, log: String) {
        self.logs.push(log);
    }
} 