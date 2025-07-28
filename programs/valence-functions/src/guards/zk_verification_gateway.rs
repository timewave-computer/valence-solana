use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke;
use borsh::{BorshDeserialize, BorshSerialize};

declare_id!("VerifyGate111111111111111111111111111111111");

#[program]
pub mod verification_gateway_guard {
    use super::*;

    /// Evaluate a ZK proof guard by delegating to the appropriate verifier
    pub fn evaluate_guard(
        ctx: Context<EvaluateGuard>,
        evaluation_context: GuardEvaluationContext,
        guard_data: ZkGuardData,
    ) -> Result<bool> {
        let vk_account = &ctx.accounts.verification_key;
        
        // 1. Optional: Check if tx submitter is whitelisted
        if guard_data.require_whitelisted_submitter {
            let submitter = ctx.accounts.payer.key();
            require!(
                vk_account.whitelisted_submitters.is_empty() || 
                vk_account.whitelisted_submitters.contains(&submitter),
                ErrorCode::SubmitterNotWhitelisted
            );
        }
        
        // 2. Validate proof system matches
        require!(
            vk_account.proof_system == guard_data.proof_system,
            ErrorCode::ProofSystemMismatch
        );
        
        // 3. Route to appropriate verifier via CPI
        let verifier_program = match guard_data.proof_system {
            ProofSystem::SP1 => ctx.accounts.sp1_verifier.key(),
            ProofSystem::Groth16 => ctx.accounts.groth16_verifier.key(),
            ProofSystem::PlonkBn254 => ctx.accounts.plonk_verifier.key(),
            ProofSystem::Halo2 => ctx.accounts.halo2_verifier.key(),
        };
        
        // 4. Prepare verification request
        let verification_request = VerificationRequest {
            vk: vk_account.verification_key.clone(),
            proof: guard_data.proof,
            public_values: guard_data.public_values,
            context: evaluation_context,
        };
        
        // 5. Build CPI instruction
        let ix = Instruction {
            program_id: verifier_program,
            accounts: vec![
                AccountMeta::new_readonly(vk_account.key(), false),
            ],
            data: verification_request.try_to_vec()?,
        };
        
        // 6. Invoke verifier
        invoke(&ix, &[vk_account.to_account_info()])?;
        
        // 7. Read result from return data
        let (program_id, data) = get_return_data()
            .ok_or(ErrorCode::NoReturnData)?;
        require!(program_id == verifier_program, ErrorCode::InvalidReturnData);
        
        let result: bool = bool::try_from_slice(&data)
            .map_err(|_| ErrorCode::InvalidReturnData)?;
        
        Ok(result)
    }

    /// Register a new verification key
    pub fn register_verification_key(
        ctx: Context<RegisterVK>,
        vk_id: u64,
        proof_system: ProofSystem,
        verification_key: Vec<u8>,
        whitelisted_submitters: Vec<Pubkey>,
    ) -> Result<()> {
        let vk_account = &mut ctx.accounts.verification_key;
        
        // Validate VK size for known proof systems
        match proof_system {
            ProofSystem::SP1 => {
                require!(verification_key.len() == 32, ErrorCode::InvalidVKSize);
            }
            _ => {
                // Other proof systems may have different sizes
                require!(verification_key.len() > 0, ErrorCode::InvalidVKSize);
            }
        }
        
        vk_account.vk_id = vk_id;
        vk_account.owner = ctx.accounts.owner.key();
        vk_account.admin = ctx.accounts.admin.key();
        vk_account.proof_system = proof_system;
        vk_account.verification_key = verification_key;
        vk_account.whitelisted_submitters = whitelisted_submitters;
        vk_account.created_at = Clock::get()?.unix_timestamp;
        vk_account.updated_at = Clock::get()?.unix_timestamp;
        
        emit!(VKRegistered {
            vk_id,
            owner: ctx.accounts.owner.key(),
            proof_system,
            vk_size: vk_account.verification_key.len(),
        });
        
        Ok(())
    }
    
    /// Update the whitelist for a verification key
    pub fn update_whitelist(
        ctx: Context<UpdateWhitelist>,
        add_submitters: Vec<Pubkey>,
        remove_submitters: Vec<Pubkey>,
    ) -> Result<()> {
        let vk_account = &mut ctx.accounts.verification_key;
        
        // Remove first to handle duplicates
        vk_account.whitelisted_submitters
            .retain(|s| !remove_submitters.contains(s));
        
        // Add new submitters
        for submitter in add_submitters {
            if !vk_account.whitelisted_submitters.contains(&submitter) {
                vk_account.whitelisted_submitters.push(submitter);
            }
        }
        
        vk_account.updated_at = Clock::get()?.unix_timestamp;
        
        emit!(WhitelistUpdated {
            vk_id: vk_account.vk_id,
            total_whitelisted: vk_account.whitelisted_submitters.len(),
        });
        
        Ok(())
    }
    
    /// Update the verification key (admin only)
    pub fn update_verification_key(
        ctx: Context<UpdateVK>,
        new_verification_key: Vec<u8>,
    ) -> Result<()> {
        let vk_account = &mut ctx.accounts.verification_key;
        
        // Validate new VK size
        match vk_account.proof_system {
            ProofSystem::SP1 => {
                require!(new_verification_key.len() == 32, ErrorCode::InvalidVKSize);
            }
            _ => {
                require!(new_verification_key.len() > 0, ErrorCode::InvalidVKSize);
            }
        }
        
        vk_account.verification_key = new_verification_key;
        vk_account.updated_at = Clock::get()?.unix_timestamp;
        
        emit!(VKUpdated {
            vk_id: vk_account.vk_id,
            proof_system: vk_account.proof_system,
            new_vk_size: vk_account.verification_key.len(),
        });
        
        Ok(())
    }
}

// ===== Accounts =====

#[derive(Accounts)]
#[instruction(vk_id: u64)]
pub struct RegisterVK<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + VerificationKey::INIT_SPACE,
        seeds = [b"vk", vk_id.to_le_bytes().as_ref(), owner.key().as_ref()],
        bump
    )]
    pub verification_key: Account<'info, VerificationKey>,
    pub owner: Signer<'info>,
    pub admin: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EvaluateGuard<'info> {
    #[account(
        seeds = [
            b"vk",
            verification_key.vk_id.to_le_bytes().as_ref(),
            verification_key.owner.as_ref()
        ],
        bump
    )]
    pub verification_key: Account<'info, VerificationKey>,
    /// CHECK: SP1 verifier program
    pub sp1_verifier: AccountInfo<'info>,
    /// CHECK: Groth16 verifier program
    pub groth16_verifier: AccountInfo<'info>,
    /// CHECK: Plonk verifier program
    pub plonk_verifier: AccountInfo<'info>,
    /// CHECK: Halo2 verifier program
    pub halo2_verifier: AccountInfo<'info>,
    /// CHECK: Transaction submitter for whitelist check
    pub payer: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct UpdateWhitelist<'info> {
    #[account(
        mut,
        has_one = admin,
        seeds = [
            b"vk",
            verification_key.vk_id.to_le_bytes().as_ref(),
            verification_key.owner.as_ref()
        ],
        bump
    )]
    pub verification_key: Account<'info, VerificationKey>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateVK<'info> {
    #[account(
        mut,
        has_one = admin,
        seeds = [
            b"vk",
            verification_key.vk_id.to_le_bytes().as_ref(),
            verification_key.owner.as_ref()
        ],
        bump
    )]
    pub verification_key: Account<'info, VerificationKey>,
    pub admin: Signer<'info>,
}

// ===== State =====

#[account]
pub struct VerificationKey {
    pub vk_id: u64,
    pub owner: Pubkey,
    pub admin: Pubkey,
    pub proof_system: ProofSystem,
    pub verification_key: Vec<u8>,
    pub whitelisted_submitters: Vec<Pubkey>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl VerificationKey {
    pub const INIT_SPACE: usize = 8 + 32 + 32 + 1 + (4 + 64) + (4 + 10 * 32) + 8 + 8;
}

// ===== Types =====

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum ProofSystem {
    SP1,
    Groth16,
    PlonkBn254,
    Halo2,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ZkGuardData {
    pub vk_id: u64,
    pub proof_system: ProofSystem,
    pub proof: Vec<u8>,
    pub public_values: Vec<u8>,
    pub require_whitelisted_submitter: bool,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct GuardEvaluationContext {
    pub session: Pubkey,
    pub owner: Pubkey,
    pub sequence_number: u64,
    pub usage_count: u64,
    pub timestamp: i64,
    pub shared_data_hash: [u8; 32],
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct VerificationRequest {
    pub vk: Vec<u8>,
    pub proof: Vec<u8>,
    pub public_values: Vec<u8>,
    pub context: GuardEvaluationContext,
}

// ===== Events =====

#[event]
pub struct VKRegistered {
    pub vk_id: u64,
    pub owner: Pubkey,
    pub proof_system: ProofSystem,
    pub vk_size: usize,
}

#[event]
pub struct WhitelistUpdated {
    pub vk_id: u64,
    pub total_whitelisted: usize,
}

#[event]
pub struct VKUpdated {
    pub vk_id: u64,
    pub proof_system: ProofSystem,
    pub new_vk_size: usize,
}

// ===== Errors =====

#[error_code]
pub enum ErrorCode {
    #[msg("Submitter not in whitelist")]
    SubmitterNotWhitelisted,
    #[msg("Proof system mismatch")]
    ProofSystemMismatch,
    #[msg("No return data from verifier")]
    NoReturnData,
    #[msg("Invalid return data from verifier")]
    InvalidReturnData,
    #[msg("Invalid verification key size")]
    InvalidVKSize,
}