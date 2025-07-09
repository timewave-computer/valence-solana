// Execution context building logic

use anchor_lang::prelude::*;
use crate::processor::execution_engine::ExecutionContext;
use crate::error::ProcessorError;

/// Builds execution contexts for capability processing
pub struct ContextBuilder {
    capability_id: String,
    session_id: Option<String>,
    session: Option<Pubkey>,
    caller: Pubkey,
    compute_limit: Option<u64>,
    input_data: Vec<u8>,
    labels: Vec<String>,
}

impl ContextBuilder {
    /// Start building a new context
    pub fn new_builder(capability_id: String, caller: Pubkey) -> Self {
        Self {
            capability_id,
            session_id: None,
            session: None,
            caller,
            compute_limit: None,
            input_data: vec![],
            labels: vec![],
        }
    }
    
    /// Add session to the context
    pub fn add_session(mut self, session_id: String, session: Pubkey) -> Self {
        self.session_id = Some(session_id);
        self.session = Some(session);
        self
    }
    
    /// Set compute limit
    pub fn set_compute_limit(mut self, limit: u64) -> Self {
        self.compute_limit = Some(limit);
        self
    }
    
    /// Add input data
    pub fn add_input_data(mut self, data: Vec<u8>) -> Self {
        self.input_data = data;
        self
    }
    
    /// Add labels
    pub fn add_labels(mut self, labels: Vec<String>) -> Self {
        self.labels = labels;
        self
    }
    
    /// Build the execution context
    pub fn build(self) -> ExecutionContext {
        let mut context = if let (Some(session_id), Some(session)) = (self.session_id, self.session) {
            ExecutionContext::with_session(self.capability_id, session_id, session, self.caller)
        } else {
            ExecutionContext::new(self.capability_id, self.caller, None)
        };
        
        if let Some(limit) = self.compute_limit {
            context = context.with_compute_limit(limit);
        }
        
        if !self.input_data.is_empty() {
            context = context.with_input_data(self.input_data);
        }
        
        context
    }
    /// Create a new execution context
    pub fn new(
        capability_id: String,
        caller: Pubkey,
    ) -> ExecutionContext {
        ExecutionContext::new(capability_id, caller, None)
    }

    /// Create execution context with session
    pub fn with_session(
        capability_id: String,
        session_id: String,
        session: Pubkey,
        caller: Pubkey,
    ) -> ExecutionContext {
        ExecutionContext::with_session(capability_id, session_id, session, caller)
    }

    /// Create execution context with compute limit
    pub fn with_compute_limit(
        capability_id: String,
        caller: Pubkey,
        compute_limit: u64,
    ) -> ExecutionContext {
        ExecutionContext::new(capability_id, caller, None)
            .with_compute_limit(compute_limit)
    }

    /// Build context from instruction accounts
    pub fn from_accounts(
        capability_id: String,
        session_account: Option<&AccountInfo>,
        caller_account: &AccountInfo,
    ) -> ExecutionContext {
        let session = session_account.map(|acc| *acc.key);
        ExecutionContext::new(capability_id, *caller_account.key, session)
    }

    /// Validate context before execution
    pub fn validate(&self, context: &ExecutionContext) -> Result<()> {
        // Validate capability ID
        if context.capability_id.is_empty() {
            return Err(ProcessorError::CapabilityInvalidId.into());
        }

        // Validate session and caller
        if context.session == Some(Pubkey::default()) || context.caller == Pubkey::default() {
            return Err(ProcessorError::ExecutionFailed.into());
        }

        Ok(())
    }

    /// Add metadata to context
    pub fn with_metadata(
        mut context: ExecutionContext,
        compute_limit: Option<u64>,
    ) -> ExecutionContext {
        context.compute_limit = compute_limit;
        context
    }
} 