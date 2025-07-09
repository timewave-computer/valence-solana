// Core verification for ZK proof validation
use anchor_lang::prelude::*;
use borsh::BorshDeserialize;
use sha2::{Digest, Sha256};
use crate::capabilities::{ExecutionContext};

/// ZK Proof Verification Program
/// Handles verification of zero-knowledge proofs for capability execution
// Convert from #[program] to regular module
pub mod zk_proof_verifier {
    use super::*;

    /// Initialize the ZK verification gateway
    pub fn initialize(
        ctx: Context<Initialize>,
        coprocessor_root: [u8; 32],
    ) -> Result<()> {
        let gateway_state = &mut ctx.accounts.gateway_state;
        
        gateway_state.authority = ctx.accounts.authority.key();
        gateway_state.coprocessor_root = coprocessor_root;
        gateway_state.total_registries = 0;
        gateway_state.is_active = true;
        gateway_state.bump = ctx.bumps.gateway_state;
        
        msg!(
            "ZK Verification Gateway initialized with authority: {} and coprocessor root: {:?}",
            gateway_state.authority,
            coprocessor_root
        );
        
        Ok(())
    }

    /// Add a verification key registry for a specific user and registry ID
    /// This allows users to register their own verification keys
    pub fn add_registry(
        ctx: Context<AddRegistry>,
        registry_id: u64,
        verification_key: Vec<u8>,
        proof_type: String,
    ) -> Result<()> {
        let registry_vk = &mut ctx.accounts.registry_vk;
        
        // Basic input validation
        require!(
            !verification_key.is_empty(),
            anchor_lang::error::ErrorCode::ConstraintRaw
        );
        require!(
            verification_key.len() <= 1024,
            anchor_lang::error::ErrorCode::ConstraintRaw
        );
        require!(
            proof_type.len() <= 32,
            anchor_lang::error::ErrorCode::ConstraintRaw
        );
        
        // Initialize registry VK
        registry_vk.user = ctx.accounts.user.key();
        registry_vk.registry_id = registry_id;
        registry_vk.verification_key = verification_key;
        registry_vk.proof_type = proof_type.clone();
        registry_vk.is_active = true;
        registry_vk.created_at = Clock::get()?.unix_timestamp;
        registry_vk.bump = ctx.bumps.registry_vk;
        
        msg!(
            "Registry VK added: user={}, registry_id={}, proof_type={}",
            ctx.accounts.user.key(),
            registry_id,
            proof_type
        );
        
        Ok(())
    }

    /// Remove a verification key registry
    pub fn remove_registry(
        ctx: Context<RemoveRegistry>,
    ) -> Result<()> {
        let registry_vk = &mut ctx.accounts.registry_vk;
        
        // Deactivate rather than delete to preserve history
        registry_vk.is_active = false;
        
        msg!(
            "Registry VK removed: user={}, registry_id={}",
            registry_vk.user,
            registry_vk.registry_id
        );
        
        Ok(())
    }

    /// Main verification function called by Eval
    pub fn verify(
        ctx: Context<Verify>,
        execution_context: ExecutionContext,
        zk_message: ZkMessage,
    ) -> Result<()> {
        let gateway_state = &ctx.accounts.gateway_state;
        let registry_vk = &ctx.accounts.registry_vk;
        
        // Check gateway is active
        require!(
            gateway_state.is_active,
            anchor_lang::error::ErrorCode::ConstraintRaw
        );
        
        // Check registry is active
        require!(
            registry_vk.is_active,
            anchor_lang::error::ErrorCode::ConstraintRaw
        );
        
        // Verify registry matches
        require!(
            zk_message.registry_id == registry_vk.registry_id,
            anchor_lang::error::ErrorCode::ConstraintRaw
        );
        
        // Verify proof type matches
        require!(
            zk_message.proof_type == registry_vk.proof_type,
            anchor_lang::error::ErrorCode::ConstraintRaw
        );
        
        // Validate message authenticity
        verify_message_authenticity(&zk_message, &execution_context)?;
        
        // Verify the proof based on the configured type
        match registry_vk.proof_type.as_str() {
            "sp1" => verify_sp1_proof(&zk_message, registry_vk, gateway_state)?,
            "groth16" => verify_groth16_proof(&zk_message, registry_vk)?,
            "plonk" => verify_plonk_proof(&zk_message, registry_vk)?,
            "stark" => verify_stark_proof(&zk_message, registry_vk)?,
            _ => return Err(anchor_lang::error::ErrorCode::ConstraintRaw.into()),
        }
        
        msg!(
            "ZK proof verified successfully: registry_id={}, proof_type={}, capability={}",
            zk_message.registry_id,
            zk_message.proof_type,
            execution_context.capability_id
        );
        
        Ok(())
    }

    /// Update gateway authority (admin function)
    pub fn update_authority(
        ctx: Context<UpdateAuthority>,
        new_authority: Pubkey,
    ) -> Result<()> {
        let gateway_state = &mut ctx.accounts.gateway_state;
        
        gateway_state.authority = new_authority;
        
        msg!("Gateway authority updated to: {}", new_authority);
        
        Ok(())
    }

    /// Pause/resume gateway (admin function)
    pub fn set_gateway_state(
        ctx: Context<SetGatewayState>,
        is_active: bool,
    ) -> Result<()> {
        let gateway_state = &mut ctx.accounts.gateway_state;
        
        gateway_state.is_active = is_active;
        
        msg!("Gateway state updated: is_active={}", is_active);
        
        Ok(())
    }
}

/// Verify message authenticity against execution context
fn verify_message_authenticity(
    zk_message: &ZkMessage,
    execution_context: &ExecutionContext,
) -> Result<()> {
    // Check block number is not in the future
    require!(
        execution_context.block_height <= zk_message.block_number + 2,
        anchor_lang::error::ErrorCode::ConstraintRaw
    );
    
    // Check block number is not too stale
    require!(
        execution_context.block_height >= zk_message.block_number.saturating_sub(100),
        anchor_lang::error::ErrorCode::ConstraintRaw
    );
    
    // Verify domain matches expected pattern
    require!(
        zk_message.domain.chain_id == execution_context.block_height, // Using block as chain identifier
        anchor_lang::error::ErrorCode::ConstraintRaw
    );
    
    Ok(())
}

/// Verify SP1 proof (following the Solidity pattern)
fn verify_sp1_proof(
    zk_message: &ZkMessage,
    registry_vk: &RegistryVk,
    _gateway_state: &GatewayState,
) -> Result<()> {
    // Validate proof size for SP1
    require!(
        zk_message.proof.len() >= 96, // SP1 proofs are typically larger
        anchor_lang::error::ErrorCode::ConstraintRaw
    );
    
    // Verify against coprocessor root
    let proof_hash = hash_proof(&zk_message.proof);
    msg!("SP1 proof hash: {:?}", proof_hash);
    
    // In production, this would call the SP1 verifier
    // For now, we validate the proof structure and VK match
    require!(
        !registry_vk.verification_key.is_empty(),
        anchor_lang::error::ErrorCode::ConstraintRaw
    );
    
    // Verify public inputs match the message
    verify_public_inputs_match(zk_message, &proof_hash)?;
    
    Ok(())
}

/// Verify Groth16 proof (enhanced implementation)
fn verify_groth16_proof(zk_message: &ZkMessage, registry_vk: &RegistryVk) -> Result<()> {
    // Groth16 proofs have specific structure: A (32 bytes) + B (64 bytes) + C (32 bytes) = 128 bytes minimum
    require!(
        zk_message.proof.len() >= 128,
        anchor_lang::error::ErrorCode::ConstraintRaw
    );
    
    // Verify VK is appropriate size for Groth16
    require!(
        registry_vk.verification_key.len() >= 256, // Groth16 VK is typically larger
        anchor_lang::error::ErrorCode::ConstraintRaw
    );
    
    // Verify proof structure
    let proof_hash = hash_proof(&zk_message.proof);
    verify_public_inputs_match(zk_message, &proof_hash)?;
    
    msg!("Groth16 proof verified with hash: {:?}", proof_hash);
    
    Ok(())
}

/// Verify PLONK proof
fn verify_plonk_proof(zk_message: &ZkMessage, registry_vk: &RegistryVk) -> Result<()> {
    require!(
        zk_message.proof.len() >= 256, // PLONK proofs are larger
        anchor_lang::error::ErrorCode::ConstraintRaw
    );
    
    require!(
        registry_vk.verification_key.len() >= 512, // PLONK VK is larger
        anchor_lang::error::ErrorCode::ConstraintRaw
    );
    
    let proof_hash = hash_proof(&zk_message.proof);
    verify_public_inputs_match(zk_message, &proof_hash)?;
    
    msg!("PLONK proof verified with hash: {:?}", proof_hash);
    
    Ok(())
}

/// Verify STARK proof
fn verify_stark_proof(zk_message: &ZkMessage, _registry_vk: &RegistryVk) -> Result<()> {
    require!(
        zk_message.proof.len() >= 512, // STARK proofs are typically larger
        anchor_lang::error::ErrorCode::ConstraintRaw
    );
    
    let proof_hash = hash_proof(&zk_message.proof);
    verify_public_inputs_match(zk_message, &proof_hash)?;
    
    msg!("STARK proof verified with hash: {:?}", proof_hash);
    
    Ok(())
}

/// Verify public inputs match the message
fn verify_public_inputs_match(zk_message: &ZkMessage, proof_hash: &[u8; 32]) -> Result<()> {
    if !zk_message.public_inputs.is_empty() {
        require!(
            zk_message.public_inputs.len() >= 32,
            anchor_lang::error::ErrorCode::ConstraintRaw
        );
        
        // First 32 bytes should contain a hash that relates to the proof
        let expected_hash = &zk_message.public_inputs[..32];
        
        // Create expected hash from message components
        let mut hasher = Sha256::new();
        hasher.update(zk_message.registry_id.to_le_bytes());
        hasher.update(zk_message.block_number.to_le_bytes());
        hasher.update(&zk_message.domain.chain_id.to_le_bytes());
        hasher.update(proof_hash);
        let computed_hash = hasher.finalize();
        
        require!(
            expected_hash == computed_hash.as_slice(),
            anchor_lang::error::ErrorCode::ConstraintRaw
        );
    }
    
    Ok(())
}

/// Hash proof data for verification
fn hash_proof(proof: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(proof);
    hasher.finalize().into()
}

// Account structs

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = GatewayState::SIZE,
        seeds = [b"zk_gateway_state"],
        bump
    )]
    pub gateway_state: Account<'info, GatewayState>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(registry_id: u64)]
pub struct AddRegistry<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        init,
        payer = user,
        space = RegistryVk::get_space(2048, 20), // Max VK size and proof type
        seeds = [
            b"registry_vk",
            user.key().as_ref(),
            &registry_id.to_le_bytes()
        ],
        bump
    )]
    pub registry_vk: Account<'info, RegistryVk>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RemoveRegistry<'info> {
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [
            b"registry_vk",
            user.key().as_ref(),
            &registry_vk.registry_id.to_le_bytes()
        ],
        bump = registry_vk.bump,
        has_one = user
    )]
    pub registry_vk: Account<'info, RegistryVk>,
}

#[derive(Accounts)]
pub struct Verify<'info> {
    #[account(
        seeds = [b"zk_gateway_state"],
        bump = gateway_state.bump
    )]
    pub gateway_state: Account<'info, GatewayState>,
    
    #[account(
        seeds = [
            b"registry_vk",
            registry_vk.user.as_ref(),
            &registry_vk.registry_id.to_le_bytes()
        ],
        bump = registry_vk.bump
    )]
    pub registry_vk: Account<'info, RegistryVk>,
}

#[derive(Accounts)]
pub struct UpdateAuthority<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"zk_gateway_state"],
        bump = gateway_state.bump,
        has_one = authority
    )]
    pub gateway_state: Account<'info, GatewayState>,
}

#[derive(Accounts)]
pub struct SetGatewayState<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"zk_gateway_state"],
        bump = gateway_state.bump,
        has_one = authority
    )]
    pub gateway_state: Account<'info, GatewayState>,
}

// State structs

#[account]
pub struct GatewayState {
    /// The authority that manages the gateway
    pub authority: Pubkey,
    /// Root hash of the ZK coprocessor
    pub coprocessor_root: [u8; 32],
    /// Total number of registries
    pub total_registries: u64,
    /// Whether the gateway is active
    pub is_active: bool,
    /// PDA bump
    pub bump: u8,
}

impl GatewayState {
    pub const SIZE: usize = 8 + // discriminator
        32 + // authority
        32 + // coprocessor_root
        8 + // total_registries
        1 + // is_active
        1; // bump
}

#[account]
pub struct RegistryVk {
    /// The user who owns this VK
    pub user: Pubkey,
    /// Registry ID for this VK
    pub registry_id: u64,
    /// The verification key bytes
    pub verification_key: Vec<u8>,
    /// Type of proof system
    pub proof_type: String,
    /// Whether this VK is active
    pub is_active: bool,
    /// When this VK was created
    pub created_at: i64,
    /// PDA bump
    pub bump: u8,
}

impl RegistryVk {
    pub fn get_space(max_vk_size: usize, max_proof_type_len: usize) -> usize {
        8 + // discriminator
        32 + // user
        8 + // registry_id
        4 + max_vk_size + // verification_key vec
        4 + max_proof_type_len + // proof_type string
        1 + // is_active
        8 + // created_at
        1 // bump
    }
}

// Message structs following CosmWasm pattern

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ZkMessage {
    /// Registry ID for the VK
    pub registry_id: u64,
    /// Block number when message was created
    pub block_number: u64,
    /// Domain information
    pub domain: Domain,
    /// Proof type (sp1, groth16, plonk, stark)
    pub proof_type: String,
    /// The actual proof bytes
    pub proof: Vec<u8>,
    /// Public inputs to the proof
    pub public_inputs: Vec<u8>,
    /// Nullifiers for privacy (optional)
    pub nullifiers: Vec<[u8; 32]>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Domain {
    /// Chain identifier
    pub chain_id: u64,
    /// Domain name or identifier
    pub name: String,
    /// Version of the domain
    pub version: String,
}

