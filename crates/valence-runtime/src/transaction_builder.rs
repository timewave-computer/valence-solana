//! Transaction construction for external signing

use crate::{Result, RuntimeError};
use serde::{Deserialize, Serialize};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    message::Message,
    pubkey::Pubkey,
    signer::Signer,
    transaction::Transaction,
};
use std::sync::Arc;
use tracing::{debug, info};

/// Unsigned transaction ready for external signing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsignedTransaction {
    /// Transaction message
    pub message: Vec<u8>,

    /// Recent blockhash used
    pub recent_blockhash: Hash,

    /// Required signers
    pub signers: Vec<Pubkey>,

    /// Transaction metadata
    pub metadata: TransactionMetadata,
}

/// Transaction metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMetadata {
    /// Transaction description
    pub description: String,

    /// Estimated compute units
    pub compute_units: Option<u32>,

    /// Priority fee (lamports per compute unit)
    pub priority_fee: Option<u64>,

    /// Transaction simulation result
    pub simulation: Option<SimulationResult>,
}

/// Transaction simulation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub success: bool,
    pub error: Option<String>,
    pub logs: Vec<String>,
    pub units_consumed: Option<u64>,
}

/// Transaction builder for constructing unsigned transactions
pub struct TransactionBuilder {
    rpc_client: Arc<RpcClient>,
    instructions: Vec<Instruction>,
    signers: Vec<Pubkey>,
    compute_units: Option<u32>,
    priority_fee: Option<u64>,
}

impl TransactionBuilder {
    /// Create a new transaction builder
    pub fn new(rpc_client: Arc<RpcClient>) -> Self {
        Self {
            rpc_client,
            instructions: Vec::new(),
            signers: Vec::new(),
            compute_units: None,
            priority_fee: None,
        }
    }

    /// Add an instruction to the transaction
    pub fn add_instruction(mut self, instruction: Instruction) -> Self {
        // Extract signers from instruction accounts
        for account in &instruction.accounts {
            if account.is_signer && !self.signers.contains(&account.pubkey) {
                self.signers.push(account.pubkey);
            }
        }

        self.instructions.push(instruction);
        self
    }

    /// Add multiple instructions
    pub fn add_instructions(mut self, instructions: Vec<Instruction>) -> Self {
        for instruction in instructions {
            self = self.add_instruction(instruction);
        }
        self
    }

    /// Set compute unit limit
    pub fn with_compute_units(mut self, units: u32) -> Self {
        self.compute_units = Some(units);
        self
    }

    /// Set priority fee
    pub fn with_priority_fee(mut self, fee: u64) -> Self {
        self.priority_fee = Some(fee);
        self
    }

    /// Build instruction from components
    pub fn instruction(
        program_id: Pubkey,
        accounts: Vec<AccountMeta>,
        data: Vec<u8>,
    ) -> Instruction {
        Instruction {
            program_id,
            accounts,
            data,
        }
    }

    /// Helper to create account meta
    pub fn account_meta(pubkey: Pubkey, is_signer: bool, is_writable: bool) -> AccountMeta {
        AccountMeta {
            pubkey,
            is_signer,
            is_writable,
        }
    }

    /// Build the unsigned transaction
    pub async fn build(mut self, description: String) -> Result<UnsignedTransaction> {
        info!("Building unsigned transaction: {}", description);

        // Add compute budget instructions if specified
        if let Some(units) = self.compute_units {
            let compute_budget_ix =
                solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(units);
            self.instructions.insert(0, compute_budget_ix);
        }

        if let Some(fee) = self.priority_fee {
            let priority_fee_ix =
                solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_price(fee);
            self.instructions.insert(0, priority_fee_ix);
        }

        // Get recent blockhash
        let recent_blockhash = self.rpc_client.get_latest_blockhash().await?;

        // Create message
        let message = Message::new_with_blockhash(
            &self.instructions,
            Some(&self.signers[0]), // Fee payer is first signer
            &recent_blockhash,
        );

        // Simulate if enabled
        let simulation = self.simulate_transaction(&message).await?;

        // Serialize message
        let message_bytes = bincode::serialize(&message)
            .map_err(|e| RuntimeError::TransactionBuildError(e.to_string()))?;

        let metadata = TransactionMetadata {
            description,
            compute_units: self.compute_units,
            priority_fee: self.priority_fee,
            simulation: Some(simulation),
        };

        Ok(UnsignedTransaction {
            message: message_bytes,
            recent_blockhash,
            signers: self.signers,
            metadata,
        })
    }

    /// Simulate transaction
    async fn simulate_transaction(&self, message: &Message) -> Result<SimulationResult> {
        debug!("Simulating transaction");

        // Create a dummy transaction for simulation
        let tx = Transaction::new_unsigned(message.clone());

        let result = self.rpc_client.simulate_transaction(&tx).await?;

        Ok(SimulationResult {
            success: result.value.err.is_none(),
            error: result.value.err.map(|e| e.to_string()),
            logs: result.value.logs.unwrap_or_default(),
            units_consumed: result.value.units_consumed,
        })
    }

    /// Helper to decode and sign a transaction (for testing)
    pub fn sign_transaction(
        unsigned: &UnsignedTransaction,
        signers: &[&dyn Signer],
    ) -> Result<Transaction> {
        let message: Message = bincode::deserialize(&unsigned.message)
            .map_err(|e| RuntimeError::TransactionBuildError(e.to_string()))?;

        let mut tx = Transaction::new_unsigned(message);
        tx.try_sign(signers, unsigned.recent_blockhash)
            .map_err(|e| RuntimeError::TransactionBuildError(e.to_string()))?;

        Ok(tx)
    }
}

/// Common instruction builders
impl TransactionBuilder {
    /// Create a transfer instruction
    pub fn transfer_instruction(from: Pubkey, to: Pubkey, lamports: u64) -> Instruction {
        solana_sdk::system_instruction::transfer(&from, &to, lamports)
    }

    /// Create account initialization instruction
    pub fn create_account_instruction(
        from: Pubkey,
        to: Pubkey,
        lamports: u64,
        space: u64,
        owner: &Pubkey,
    ) -> Instruction {
        solana_sdk::system_instruction::create_account(&from, &to, lamports, space, owner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transaction_builder() {
        let rpc_client = Arc::new(RpcClient::new(
            "https://api.mainnet-beta.solana.com".to_string(),
        ));
        let builder = TransactionBuilder::new(rpc_client);

        let from = Pubkey::new_unique();
        let to = Pubkey::new_unique();

        let tx = builder
            .add_instruction(TransactionBuilder::transfer_instruction(from, to, 1000))
            .with_compute_units(200_000)
            .with_priority_fee(1000)
            .build("Test transfer".to_string())
            .await;

        assert!(tx.is_ok());
        let unsigned = tx.unwrap();
        assert_eq!(unsigned.signers.len(), 1);
        assert_eq!(unsigned.signers[0], from);
    }

    #[test]
    fn test_instruction_building() {
        let program_id = Pubkey::new_unique();
        let account1 = Pubkey::new_unique();
        let account2 = Pubkey::new_unique();

        let instruction = TransactionBuilder::instruction(
            program_id,
            vec![
                TransactionBuilder::account_meta(account1, true, true),
                TransactionBuilder::account_meta(account2, false, false),
            ],
            vec![1, 2, 3, 4],
        );

        assert_eq!(instruction.program_id, program_id);
        assert_eq!(instruction.accounts.len(), 2);
        assert_eq!(instruction.data, vec![1, 2, 3, 4]);
    }
}
