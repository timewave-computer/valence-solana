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

impl InitializeInstruction {
    pub fn data(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.push(0); // Instruction discriminator for Initialize
        let mut param_data = self.params.try_to_vec()?;
        buf.append(&mut param_data);
        Ok(buf)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TransferTokenInstruction {
    pub params: TransferTokenParams,
}

impl TransferTokenInstruction {
    pub fn data(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.push(1); // Instruction discriminator for TransferToken
        let mut param_data = self.params.try_to_vec()?;
        buf.append(&mut param_data);
        Ok(buf)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TransferSolInstruction {
    pub params: TransferSolParams,
}

impl TransferSolInstruction {
    pub fn data(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.push(2); // Instruction discriminator for TransferSol
        let mut param_data = self.params.try_to_vec()?;
        buf.append(&mut param_data);
        Ok(buf)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct BatchTransferInstruction {
    pub params: BatchTransferParams,
}

impl BatchTransferInstruction {
    pub fn data(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.push(3); // Instruction discriminator for BatchTransfer
        let mut param_data = self.params.try_to_vec()?;
        buf.append(&mut param_data);
        Ok(buf)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TransferWithAuthorityInstruction {
    pub params: TransferWithAuthorityParams,
}

impl TransferWithAuthorityInstruction {
    pub fn data(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.push(4); // Instruction discriminator for TransferWithAuthority
        let mut param_data = self.params.try_to_vec()?;
        buf.append(&mut param_data);
        Ok(buf)
    }
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
                let mut param_data = params.try_to_vec()?;
                buf.append(&mut param_data);
            },
            TokenTransferInstruction::TransferToken(params) => {
                buf.push(1); // Instruction discriminator for TransferToken
                let mut param_data = params.try_to_vec()?;
                buf.append(&mut param_data);
            },
            TokenTransferInstruction::TransferSol(params) => {
                buf.push(2); // Instruction discriminator for TransferSol
                let mut param_data = params.try_to_vec()?;
                buf.append(&mut param_data);
            },
            TokenTransferInstruction::BatchTransfer(params) => {
                buf.push(3); // Instruction discriminator for BatchTransfer
                let mut param_data = params.try_to_vec()?;
                buf.append(&mut param_data);
            },
            TokenTransferInstruction::TransferWithAuthority(params) => {
                buf.push(4); // Instruction discriminator for TransferWithAuthority
                let mut param_data = params.try_to_vec()?;
                buf.append(&mut param_data);
            },
        }
        Ok(buf)
    }
} 