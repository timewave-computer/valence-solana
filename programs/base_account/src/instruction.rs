use anchor_lang::prelude::*;
use crate::instructions::initialize::InitializeParams;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum BaseAccountInstruction {
    Initialize(InitializeParams),
    RegisterLibrary(Pubkey),
    ApproveLibrary(Pubkey),
    RevokeLibrary(Pubkey),
    CreateTokenAccount(Pubkey),
    CloseTokenAccount(Pubkey),
    ExecuteInstruction,
    TransferTokens,
}

impl BaseAccountInstruction {
    pub fn data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        match self {
            Self::Initialize(params) => {
                data.extend_from_slice(&[0u8]); // Initialize discriminator
                data.extend_from_slice(&anchor_lang::AnchorSerialize::try_to_vec(params)
                    .expect("Failed to serialize InitializeParams"));
            }
            Self::RegisterLibrary(program_id) => {
                data.extend_from_slice(&[1u8]); // RegisterLibrary discriminator
                data.extend_from_slice(&anchor_lang::AnchorSerialize::try_to_vec(program_id)
                    .expect("Failed to serialize program_id"));
            }
            Self::ApproveLibrary(library) => {
                data.extend_from_slice(&[2u8]); // ApproveLibrary discriminator
                data.extend_from_slice(&anchor_lang::AnchorSerialize::try_to_vec(library)
                    .expect("Failed to serialize library"));
            }
            Self::RevokeLibrary(library) => {
                data.extend_from_slice(&[3u8]); // RevokeLibrary discriminator
                data.extend_from_slice(&anchor_lang::AnchorSerialize::try_to_vec(library)
                    .expect("Failed to serialize library"));
            }
            Self::CreateTokenAccount(mint) => {
                data.extend_from_slice(&[4u8]); // CreateTokenAccount discriminator
                data.extend_from_slice(&anchor_lang::AnchorSerialize::try_to_vec(mint)
                    .expect("Failed to serialize mint"));
            }
            Self::CloseTokenAccount(token_account) => {
                data.extend_from_slice(&[5u8]); // CloseTokenAccount discriminator
                data.extend_from_slice(&anchor_lang::AnchorSerialize::try_to_vec(token_account)
                    .expect("Failed to serialize token_account"));
            }
            Self::ExecuteInstruction => {
                data.extend_from_slice(&[6u8]); // ExecuteInstruction discriminator
            }
            Self::TransferTokens => {
                data.extend_from_slice(&[7u8]); // TransferTokens discriminator
            }
        }
        data
    }
} 