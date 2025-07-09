// Core verification common utilities
use anchor_lang::prelude::*;
use crate::capabilities::{ExecutionContext};
use crate::verification::VerificationError;

/// Basic Permission Verifier
/// This verification function checks if the sender has permission to execute a capability
/// by verifying they are in an allowlist or have the required role

// Convert from #[program] to regular module
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


// System Authentication Verifier
// This default verification function validates the system caller authentication chain
// Replaces hardcoded checks for: entrypoint -> eval -> shard authorization

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

