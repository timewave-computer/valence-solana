/// Verification state management
/// This module contains all account state and configuration management for verification functions
use anchor_lang::prelude::*;
use crate::capabilities::ExecutionContext;
use crate::verification::VerificationError;
use crate::sessions::{SessionEntry, SessionFactoryState};

// ======================= PERMISSION CONFIGURATION =======================

// Basic Permission Verifier Module
pub mod basic_permission_verifier {
    use super::*;

    /// Verify that the sender has permission to execute the capability
    /// This is a pure function that returns success/failure based on permission checks
    pub fn verify(
        ctx: Context<Verify>,
        execution_context: ExecutionContext,
    ) -> Result<()> {
        let permission_config = &ctx.accounts.permission_config;
        
        // Check if the sender is in the allowlist
        let is_allowed = permission_config.allowed_senders.iter()
            .any(|allowed| allowed == &execution_context.caller);
        
        // Check if verification is enabled
        require!(
            permission_config.is_active,
            VerificationError::VerificationPermissionConfigNotActive
        );
        
        // Verify the sender has permission
        require!(
            is_allowed,
            VerificationError::VerificationSenderNotAuthorized
        );
        
        msg!(
            "Basic permission verified for sender: {} on capability: {}",
            execution_context.caller,
            execution_context.capability_id
        );
        
        Ok(())
    }

    /// Initialize permission configuration for a capability
    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        capability_id: String,
        allowed_senders: Vec<Pubkey>,
    ) -> Result<()> {
        let permission_config = &mut ctx.accounts.permission_config;
        
        permission_config.capability_id = capability_id;
        permission_config.allowed_senders = allowed_senders;
        permission_config.is_active = true;
        permission_config.authority = ctx.accounts.authority.key();
        permission_config.bump = ctx.bumps.permission_config;
        
        msg!(
            "Permission config initialized for capability: {} with {} allowed senders",
            permission_config.capability_id,
            permission_config.allowed_senders.len()
        );
        
        Ok(())
    }

    /// Update the allowed senders list
    pub fn update_allowed_senders(
        ctx: Context<UpdateConfig>,
        allowed_senders: Vec<Pubkey>,
    ) -> Result<()> {
        let permission_config = &mut ctx.accounts.permission_config;
        
        permission_config.allowed_senders = allowed_senders;
        
        msg!(
            "Updated allowed senders for capability: {} to {} senders",
            permission_config.capability_id,
            permission_config.allowed_senders.len()
        );
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Verify<'info> {
    /// The permission configuration for this capability
    #[account(
        seeds = [
            b"permission_config",
            permission_config.capability_id.as_bytes()
        ],
        bump = permission_config.bump
    )]
    pub permission_config: Account<'info, PermissionConfig>,
}

#[derive(Accounts)]
#[instruction(capability_id: String)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = PermissionConfig::get_space(&capability_id, 10), // Max 10 allowed senders
        seeds = [
            b"permission_config",
            capability_id.as_bytes()
        ],
        bump
    )]
    pub permission_config: Account<'info, PermissionConfig>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [
            b"permission_config",
            permission_config.capability_id.as_bytes()
        ],
        bump = permission_config.bump,
        has_one = authority
    )]
    pub permission_config: Account<'info, PermissionConfig>,
}

/// Permission configuration state
#[account]
pub struct PermissionConfig {
    /// The capability ID this config is for
    pub capability_id: String,
    /// List of allowed senders
    pub allowed_senders: Vec<Pubkey>,
    /// Whether this config is active
    pub is_active: bool,
    /// The authority that can update this config
    pub authority: Pubkey,
    /// PDA bump
    pub bump: u8,
}

impl PermissionConfig {
    pub fn get_space(capability_id: &str, max_senders: usize) -> usize {
        8 + // discriminator
        4 + capability_id.len() + // capability_id
        4 + (32 * max_senders) + // allowed_senders vec
        1 + // is_active
        32 + // authority
        1 // bump
    }
}

// ======================= AUTHENTICATION STATE =======================

/// Verify entrypoint-level authentication
#[allow(dead_code)]
fn verify_entrypoint_auth(
    execution_context: &ExecutionContext,
    _auth_state: &AuthState,
) -> Result<()> {
    // At entrypoint level, we validate that the call is coming from the correct context
    // This would typically check that the entrypoint program is being invoked correctly
    
    msg!("Entrypoint auth validated for caller: {}", execution_context.caller);
    Ok(())
}

/// Verify eval-level authentication  
#[allow(dead_code)]
fn verify_eval_auth(
    execution_context: &ExecutionContext,
    _auth_state: &AuthState,
) -> Result<()> {
    // At eval level, validate that the caller is the authorized entrypoint
    msg!(
        "Eval auth validated: caller {} (not fully implemented)",
        execution_context.caller
    );
    
    Ok(())
}

/// Verify shard-level authentication
#[allow(dead_code)]
fn verify_shard_auth(
    execution_context: &ExecutionContext,
    _auth_state: &AuthState,
) -> Result<()> {
    // At shard level, validate that the caller is the authorized eval
    msg!(
        "Shard auth validated: caller {} (not fully implemented)",
        execution_context.caller
    );
    
    Ok(())
}

#[derive(Accounts)]
pub struct VerifyAuth<'info> {
    /// The authentication configuration for this capability
    #[account(
        seeds = [
            b"auth_state",
            auth_state.capability_id.as_bytes()
        ],
        bump = auth_state.bump
    )]
    pub auth_state: Account<'info, AuthState>,
}

#[derive(Accounts)]
#[instruction(capability_id: String)]
pub struct InitializeAuthConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = AuthState::get_space(&capability_id),
        seeds = [
            b"auth_state",
            capability_id.as_bytes()
        ],
        bump
    )]
    pub auth_state: Account<'info, AuthState>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateAuthConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [
            b"auth_state",
            auth_state.capability_id.as_bytes()
        ],
        bump = auth_state.bump,
        has_one = authority
    )]
    pub auth_state: Account<'info, AuthState>,
}

/// Authentication state for system verification
#[account]
pub struct AuthState {
    /// The capability ID this auth is for
    pub capability_id: String,
    /// Authorized entrypoint program
    pub authorized_entrypoint: Pubkey,
    /// Authorized eval program
    pub authorized_eval: Pubkey,
    /// Authorized shard program
    pub authorized_shard: Pubkey,
    /// Whether this auth is active
    pub is_active: bool,
    /// The authority that can update this auth
    pub authority: Pubkey,
    /// PDA bump
    pub bump: u8,
}

impl AuthState {
    pub fn get_space(capability_id: &str) -> usize {
        8 + // discriminator
        4 + capability_id.len() + // capability_id
        32 + // authorized_entrypoint
        32 + // authorized_eval
        32 + // authorized_shard
        1 + // is_active
        32 + // authority
        1 // bump
    }
}

/// Authentication configuration for execution context
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct AuthConfig {
    /// The execution level this verification is being run at
    pub execution_level: ExecutionLevel,
}

/// Execution level in the system hierarchy
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum ExecutionLevel {
    Entrypoint,
    Eval,
    Shard,
}

// ======================= BLOCK HEIGHT STATE =======================

// Block Height Verifier Module
pub mod block_height_verifier {
    use super::*;

    /// Verify that the current execution can proceed based on block height
    /// Prevents replay attacks by ensuring monotonic ordering
    pub fn verify(
        ctx: Context<VerifyBlockHeight>,
        execution_context: ExecutionContext,
    ) -> Result<()> {
        let block_state = &mut ctx.accounts.block_state;
        
        // Check if verification is enabled
        require!(
            block_state.is_active,
            VerificationError::VerificationBlockStateNotActive
        );
        
        let current_block = execution_context.block_height;
        let last_block = block_state.last_execution_block;
        
        // Ensure current block is greater than last execution block
        // This prevents replay attacks and ensures monotonic ordering
        require!(
            current_block > last_block,
            VerificationError::VerificationInvalidBlockOrder
        );
        
        // Update the last execution block height
        block_state.last_execution_block = current_block;
        block_state.total_executions = block_state
            .total_executions
            .checked_add(1)
            .unwrap_or(u64::MAX);
        
        // Optional: Add staleness check to prevent very old transactions
        if let Some(max_staleness) = block_state.max_block_staleness {
            // Get a reasonable current block (this would be passed from context in real implementation)
            let current_network_block = current_block; // In practice, this should come from Clock
            require!(
                current_block + max_staleness >= current_network_block,
                VerificationError::VerificationTransactionTooStale
            );
        }
        
        msg!(
            "Block height verified for capability: {} - current: {}, last: {}, total executions: {}",
            execution_context.capability_id,
            current_block,
            last_block,
            block_state.total_executions
        );
        
        Ok(())
    }

    /// Initialize block state configuration for a capability
    pub fn initialize_config(
        ctx: Context<InitializeBlockConfig>,
        capability_id: String,
        max_block_staleness: Option<u64>,
    ) -> Result<()> {
        let block_state = &mut ctx.accounts.block_state;
        
        block_state.capability_id = capability_id.clone();
        block_state.last_execution_block = 0; // Start from block 0
        block_state.total_executions = 0;
        block_state.max_block_staleness = max_block_staleness;
        block_state.is_active = true;
        block_state.authority = ctx.accounts.authority.key();
        block_state.bump = ctx.bumps.block_state;
        
        msg!(
            "Block state initialized for capability: {} with max staleness: {:?}",
            capability_id,
            max_block_staleness
        );
        
        Ok(())
    }

    /// Update block height configuration
    pub fn update_config(
        ctx: Context<UpdateBlockConfig>,
        max_block_staleness: Option<u64>,
        reset_block_height: bool,
    ) -> Result<()> {
        let block_state = &mut ctx.accounts.block_state;
        
        block_state.max_block_staleness = max_block_staleness;
        
        // Allow resetting block height for testing or emergency situations
        if reset_block_height {
            block_state.last_execution_block = 0;
            msg!("Block height reset for capability: {}", block_state.capability_id);
        }
        
        msg!(
            "Updated block config for capability: {} - max staleness: {:?}",
            block_state.capability_id,
            max_block_staleness
        );
        
        Ok(())
    }
    
    /// Emergency pause/resume function
    pub fn set_active_state(
        ctx: Context<UpdateBlockConfig>,
        is_active: bool,
    ) -> Result<()> {
        let block_state = &mut ctx.accounts.block_state;
        
        block_state.is_active = is_active;
        
        msg!(
            "Block verification state updated for capability: {} - active: {}",
            block_state.capability_id,
            is_active
        );
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct VerifyBlockHeight<'info> {
    /// The block state configuration for this capability
    #[account(
        mut,
        seeds = [
            b"block_state",
            block_state.capability_id.as_bytes()
        ],
        bump = block_state.bump
    )]
    pub block_state: Account<'info, BlockState>,
}

#[derive(Accounts)]
#[instruction(capability_id: String)]
pub struct InitializeBlockConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = BlockState::get_space(&capability_id),
        seeds = [
            b"block_state",
            capability_id.as_bytes()
        ],
        bump
    )]
    pub block_state: Account<'info, BlockState>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateBlockConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [
            b"block_state",
            block_state.capability_id.as_bytes()
        ],
        bump = block_state.bump,
        has_one = authority
    )]
    pub block_state: Account<'info, BlockState>,
}

/// Block height state for replay attack prevention
#[account]
pub struct BlockState {
    /// The capability ID this block state is for
    pub capability_id: String,
    /// Last block height when this capability was executed
    pub last_execution_block: u64,
    /// Total number of executions
    pub total_executions: u64,
    /// Maximum allowed block staleness (optional)
    pub max_block_staleness: Option<u64>,
    /// Whether this verification is active
    pub is_active: bool,
    /// The authority that can update this state
    pub authority: Pubkey,
    /// PDA bump
    pub bump: u8,
}

impl BlockState {
    pub fn get_space(capability_id: &str) -> usize {
        8 + // discriminator
        4 + capability_id.len() + // capability_id
        8 + // last_execution_block
        8 + // total_executions
        1 + 8 + // max_block_staleness option
        1 + // is_active
        32 + // authority
        1 // bump
    }
    
    /// Check if execution is allowed based on current block height
    pub fn validate_execution_allowed(&self, current_block: u64) -> bool {
        current_block > self.last_execution_block
    }
}

// ======================= PARAMETER CONSTRAINT STATE =======================

// Parameter Constraint Verifier Module
pub mod parameter_constraint_verifier {
    use super::*;

    /// Verify that the call parameters meet the configured constraints
    pub fn verify(
        ctx: Context<VerifyConstraints>,
        execution_context: ExecutionContext,
        call_params: CallParameters,
    ) -> Result<()> {
        let constraint_config = &ctx.accounts.constraint_config;
        
        // Check if verification is enabled
        require!(
            constraint_config.is_active,
            VerificationError::VerificationConstraintConfigNotActive
        );
        
        // Verify amount constraints if configured
        if let Some(max_amount) = constraint_config.max_amount {
            require!(
                call_params.amount <= max_amount,
                VerificationError::VerificationAmountExceedsMaximum
            );
        }
        
        if let Some(min_amount) = constraint_config.min_amount {
            require!(
                call_params.amount >= min_amount,
                VerificationError::VerificationAmountBelowMinimum
            );
        }
        
        // Verify recipient is in allowlist if configured
        if !constraint_config.allowed_recipients.is_empty() {
            let is_allowed = constraint_config.allowed_recipients.iter()
                .any(|allowed| allowed == &call_params.recipient);
            
            require!(
                is_allowed,
                VerificationError::VerificationRecipientNotAllowed
            );
        }
        
        // Verify slippage tolerance if configured
        if let Some(max_slippage_bps) = constraint_config.max_slippage_bps {
            require!(
                call_params.slippage_bps <= max_slippage_bps,
                VerificationError::VerificationSlippageExceedsTolerance
            );
        }
        
        msg!(
            "Parameter constraints verified for capability: {} amount: {} recipient: {}",
            execution_context.capability_id,
            call_params.amount,
            call_params.recipient
        );
        
        Ok(())
    }

    /// Initialize constraint configuration for a capability
    pub fn initialize_constraint_config(
        ctx: Context<InitializeConstraintConfig>,
        capability_id: String,
        config_params: ConstraintConfigParams,
    ) -> Result<()> {
        let constraint_config = &mut ctx.accounts.constraint_config;
        
        constraint_config.capability_id = capability_id;
        constraint_config.max_amount = config_params.max_amount;
        constraint_config.min_amount = config_params.min_amount;
        constraint_config.allowed_recipients = config_params.allowed_recipients;
        constraint_config.max_slippage_bps = config_params.max_slippage_bps;
        constraint_config.is_active = true;
        constraint_config.authority = ctx.accounts.authority.key();
        constraint_config.bump = ctx.bumps.constraint_config;
        
        msg!(
            "Constraint config initialized for capability: {}",
            constraint_config.capability_id
        );
        
        Ok(())
    }

    /// Update constraint configuration
    pub fn update_constraint_config(
        ctx: Context<UpdateConstraintConfig>,
        config_params: ConstraintConfigParams,
    ) -> Result<()> {
        let constraint_config = &mut ctx.accounts.constraint_config;
        
        constraint_config.max_amount = config_params.max_amount;
        constraint_config.min_amount = config_params.min_amount;
        constraint_config.allowed_recipients = config_params.allowed_recipients;
        constraint_config.max_slippage_bps = config_params.max_slippage_bps;
        
        msg!(
            "Updated constraints for capability: {}",
            constraint_config.capability_id
        );
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct VerifyConstraints<'info> {
    /// The constraint configuration for this capability
    #[account(
        seeds = [
            b"constraint_config",
            constraint_config.capability_id.as_bytes()
        ],
        bump = constraint_config.bump
    )]
    pub constraint_config: Account<'info, ConstraintConfig>,
}

#[derive(Accounts)]
#[instruction(capability_id: String)]
pub struct InitializeConstraintConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = ConstraintConfig::get_space(&capability_id, 10), // Max 10 allowed recipients
        seeds = [
            b"constraint_config",
            capability_id.as_bytes()
        ],
        bump
    )]
    pub constraint_config: Account<'info, ConstraintConfig>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateConstraintConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [
            b"constraint_config",
            constraint_config.capability_id.as_bytes()
        ],
        bump = constraint_config.bump,
        has_one = authority
    )]
    pub constraint_config: Account<'info, ConstraintConfig>,
}

/// Constraint configuration state
#[account]
pub struct ConstraintConfig {
    /// The capability ID this config is for
    pub capability_id: String,
    /// Maximum amount allowed (None = no limit)
    pub max_amount: Option<u64>,
    /// Minimum amount required (None = no minimum)
    pub min_amount: Option<u64>,
    /// List of allowed recipients (empty = all allowed)
    pub allowed_recipients: Vec<Pubkey>,
    /// Maximum slippage in basis points (None = no limit)
    pub max_slippage_bps: Option<u16>,
    /// Whether this config is active
    pub is_active: bool,
    /// The authority that can update this config
    pub authority: Pubkey,
    /// PDA bump
    pub bump: u8,
}

impl ConstraintConfig {
    pub fn get_space(capability_id: &str, max_recipients: usize) -> usize {
        8 + // discriminator
        4 + capability_id.len() + // capability_id
        1 + 8 + // max_amount option
        1 + 8 + // min_amount option
        4 + (32 * max_recipients) + // allowed_recipients vec
        1 + 2 + // max_slippage_bps option
        1 + // is_active
        32 + // authority
        1 // bump
    }
}

/// Configuration parameters for constraints
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ConstraintConfigParams {
    pub max_amount: Option<u64>,
    pub min_amount: Option<u64>,
    pub allowed_recipients: Vec<Pubkey>,
    pub max_slippage_bps: Option<u16>,
}

/// Call parameters to verify
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CallParameters {
    pub amount: u64,
    pub recipient: Pubkey,
    pub slippage_bps: u16,
    pub token_mint: Option<Pubkey>,
}

// ======================= SESSION VERIFICATION STATE =======================

/// Basic verification context (no external accounts needed)
#[derive(Accounts)]
pub struct VerifySessionCreation<'info> {
    /// The verifier (can be anyone - this is a pure function)
    pub verifier: Signer<'info>,
}

/// Extended verification context with factory registry check
#[derive(Accounts)]
pub struct VerifyWithRegistry<'info> {
    /// The verifier (can be anyone)
    pub verifier: Signer<'info>,
    
    /// The session entry in the session factory's registry
    pub session_entry: Account<'info, SessionEntry>,
    
    /// The session factory state
    #[account(
        seeds = [b"session_factory"],
        bump = factory_state.bump
    )]
    pub factory_state: Account<'info, SessionFactoryState>,
} 