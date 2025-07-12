//! Verifier - Routes verification predicates to verifiers
//! 
//! Simple routing layer that maps verifier_label → verifier_program
//! No verification logic here - pure routing only

use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111113");

pub mod state;
pub mod instructions;
pub mod error;

pub use state::*;
pub use instructions::*;
pub use error::*;

#[program]
pub mod valence_verifier {
    use super::*;

    /// Register a new verifier
    pub fn register_verifier(
        ctx: Context<RegisterVerifier>,
        label: String,
        program: Pubkey,
    ) -> Result<()> {
        instructions::register_verifier(ctx, label, program)
    }

    /// Update verifier program
    pub fn update_verifier(
        ctx: Context<UpdateVerifier>,
        label: String,
        new_program: Pubkey,
    ) -> Result<()> {
        instructions::update_verifier(ctx, label, new_program)
    }

    /// Route verification request to appropriate verifier
    pub fn verify_predicate(
        ctx: Context<VerifyPredicate>,
        label: String,
        predicate_data: Vec<u8>,
        context: Vec<u8>,
    ) -> Result<()> {
        instructions::verify_predicate(ctx, label, predicate_data, context)
    }
}