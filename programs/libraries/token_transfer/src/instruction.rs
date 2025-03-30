use anchor_lang::prelude::*;
use crate::instructions::{
    initialize::InitializeParams,
    transfer_token::TransferTokenParams,
    transfer_sol::TransferSolParams,
    batch_transfer::BatchTransferParams,
    transfer_with_authority::TransferWithAuthorityParams,
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeInstruction {
    pub params: InitializeParams,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TransferTokenInstruction {
    pub params: TransferTokenParams,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TransferSolInstruction {
    pub params: TransferSolParams,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct BatchTransferInstruction {
    pub params: BatchTransferParams,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TransferWithAuthorityInstruction {
    pub params: TransferWithAuthorityParams,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum TokenTransferInstruction {
    Initialize(InitializeParams),
    TransferToken(TransferTokenParams),
    TransferSol(TransferSolParams),
    BatchTransfer(BatchTransferParams),
    TransferWithAuthority(TransferWithAuthorityParams),
}

impl TokenTransferInstruction {
    pub fn data(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        match self {
            TokenTransferInstruction::Initialize(params) => {
                buf.push(0); // Instruction discriminator for Initialize
                params.try_to_vec()?.extend_from_slice(&mut buf);
            },
            TokenTransferInstruction::TransferToken(params) => {
                buf.push(1); // Instruction discriminator for TransferToken
                params.try_to_vec()?.extend_from_slice(&mut buf);
            },
            TokenTransferInstruction::TransferSol(params) => {
                buf.push(2); // Instruction discriminator for TransferSol
                params.try_to_vec()?.extend_from_slice(&mut buf);
            },
            TokenTransferInstruction::BatchTransfer(params) => {
                buf.push(3); // Instruction discriminator for BatchTransfer
                params.try_to_vec()?.extend_from_slice(&mut buf);
            },
            TokenTransferInstruction::TransferWithAuthority(params) => {
                buf.push(4); // Instruction discriminator for TransferWithAuthority
                params.try_to_vec()?.extend_from_slice(&mut buf);
            },
        }
        Ok(buf)
    }
} 