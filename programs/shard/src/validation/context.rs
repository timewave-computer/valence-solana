//! Context isolation logic

use anchor_lang::prelude::*;
use crate::ShardError;

/// Execution context for isolated capability execution
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Session ID this context belongs to
    pub session_id: Pubkey,
    /// Capabilities available in this context
    pub capabilities: Vec<String>,
    /// Current state hash
    pub state_hash: [u8; 32],
    /// Accounts accessible in this context
    pub accessible_accounts: Vec<Pubkey>,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(
        session_id: Pubkey,
        capabilities: Vec<String>,
        state_hash: [u8; 32],
    ) -> Self {
        Self {
            session_id,
            capabilities,
            state_hash,
            accessible_accounts: vec![],
        }
    }
    
    /// Check if a capability is available
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }
    
    /// Verify account access is allowed
    pub fn verify_account_access(&self, account: &Pubkey) -> Result<()> {
        require!(
            self.accessible_accounts.contains(account),
            ShardError::ContextIsolationViolation
        );
        Ok(())
    }
    
    /// Add an account to accessible list
    pub fn grant_account_access(&mut self, account: Pubkey) {
        if !self.accessible_accounts.contains(&account) {
            self.accessible_accounts.push(account);
        }
    }
}

/// Validate that an operation respects context isolation
pub fn validate_operation_context(
    context: &ExecutionContext,
    required_capability: &str,
    accessed_accounts: &[Pubkey],
) -> Result<()> {
    // Check capability
    require!(
        context.has_capability(required_capability),
        ShardError::CapabilityNotGranted
    );
    
    // Check account access
    for account in accessed_accounts {
        context.verify_account_access(account)?;
    }
    
    Ok(())
}

/// Create an isolated context for function execution
pub fn create_isolated_context(
    session_id: Pubkey,
    capabilities: Vec<String>,
    state_hash: [u8; 32],
    allowed_accounts: Vec<Pubkey>,
) -> ExecutionContext {
    let mut context = ExecutionContext::new(session_id, capabilities, state_hash);
    
    // Grant access to allowed accounts
    for account in allowed_accounts {
        context.grant_account_access(account);
    }
    
    context
}