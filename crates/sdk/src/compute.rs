use crate::{Result, SdkError};
use solana_sdk::instruction::Instruction;

/// Compute budget analysis for transactions
pub struct ComputeAnalyzer {
    base_cost: u64,
    per_instruction: u64,
    per_account: u64,
    per_signer: u64,
}

impl Default for ComputeAnalyzer {
    fn default() -> Self {
        Self {
            base_cost: 5_000,
            per_instruction: 10_000,
            per_account: 2_000,
            per_signer: 1_000,
        }
    }
}

impl ComputeAnalyzer {
    /// Create a new compute analyzer
    pub fn new() -> Self {
        Self::default()
    }

    /// Estimate compute units for instructions
    pub fn estimate(&self, instructions: &[Instruction], signers: usize) -> u64 {
        let mut total = self.base_cost;

        for ix in instructions {
            total += self.per_instruction;
            total += (ix.accounts.len() as u64) * self.per_account;
        }

        total += (signers as u64) * self.per_signer;
        total
    }

    /// Check if estimated compute exceeds limit
    pub fn check_limit(&self, estimated: u64, limit: u64) -> Result<()> {
        if estimated > limit {
            Err(SdkError::ComputeBudgetExceeded { estimated, limit })
        } else {
            Ok(())
        }
    }

    /// Get recommendations for optimization
    pub fn recommendations(&self, estimated: u64) -> Vec<String> {
        let mut recommendations = Vec::new();

        if estimated > 200_000 {
            recommendations.push("Consider splitting into multiple transactions".to_string());
        }

        if estimated > 100_000 {
            recommendations.push("Consider reducing the number of accounts".to_string());
        }

        recommendations
    }
}

/// Helper to create compute budget instructions
pub fn compute_budget_instructions(units: u32, additional_fee: u64) -> Vec<Instruction> {
    use solana_sdk::compute_budget::ComputeBudgetInstruction;

    vec![
        ComputeBudgetInstruction::set_compute_unit_limit(units),
        ComputeBudgetInstruction::set_compute_unit_price(additional_fee),
    ]
}