// Transaction optimization utilities for Valence Solana programs
// Helps optimize transaction sizes, batching strategies, and instruction packing

use anchor_lang::prelude::*;
use solana_program::instruction::Instruction;

/// Solana transaction size limits
pub const MAX_TRANSACTION_SIZE: usize = 1232; // Maximum transaction size in bytes
pub const TRANSACTION_HEADER_SIZE: usize = 64; // Approximate header size
pub const SIGNATURE_SIZE: usize = 64;
pub const INSTRUCTION_HEADER_SIZE: usize = 32; // Approximate per-instruction overhead
pub const ACCOUNT_META_SIZE: usize = 34; // AccountMeta size (32 + 1 + 1)

/// Transaction optimization configuration
#[derive(Debug, Clone)]
pub struct TransactionOptimizationConfig {
    pub max_instructions_per_tx: usize,
    pub max_accounts_per_tx: usize,
    pub target_size_percentage: u8, // Target % of max transaction size
    pub enable_compression: bool,
    pub enable_account_deduplication: bool,
}

impl Default for TransactionOptimizationConfig {
    fn default() -> Self {
        Self {
            max_instructions_per_tx: 8,
            max_accounts_per_tx: 32,
            target_size_percentage: 85, // Use 85% of max size for safety
            enable_compression: true,
            enable_account_deduplication: true,
        }
    }
}

/// Transaction size estimator
pub struct TransactionSizeEstimator;

impl TransactionSizeEstimator {
    /// Estimate transaction size for a set of instructions
    pub fn estimate_transaction_size(
        instructions: &[Instruction],
        signer_count: usize,
    ) -> usize {
        let mut total_size = TRANSACTION_HEADER_SIZE;
        
        // Add signature size
        total_size += signer_count * SIGNATURE_SIZE;
        
        // Add instruction sizes
        for instruction in instructions {
            total_size += Self::estimate_instruction_size(instruction);
        }
        
        total_size
    }
    
    /// Estimate size of a single instruction
    pub fn estimate_instruction_size(instruction: &Instruction) -> usize {
        let mut size = INSTRUCTION_HEADER_SIZE;
        
        // Add account metas size
        size += instruction.accounts.len() * ACCOUNT_META_SIZE;
        
        // Add instruction data size
        size += instruction.data.len();
        
        size
    }
    
    /// Check if transaction would exceed size limit
    pub fn would_exceed_limit(
        instructions: &[Instruction],
        signer_count: usize,
        config: &TransactionOptimizationConfig,
    ) -> bool {
        let estimated_size = Self::estimate_transaction_size(instructions, signer_count);
        let target_size = (MAX_TRANSACTION_SIZE * config.target_size_percentage as usize) / 100;
        
        estimated_size > target_size
    }
    
    /// Calculate remaining space in transaction
    pub fn remaining_space(
        current_instructions: &[Instruction],
        signer_count: usize,
        config: &TransactionOptimizationConfig,
    ) -> usize {
        let current_size = Self::estimate_transaction_size(current_instructions, signer_count);
        let target_size = (MAX_TRANSACTION_SIZE * config.target_size_percentage as usize) / 100;
        
        target_size.saturating_sub(current_size)
    }
}

/// Transaction batch optimizer
pub struct TransactionBatchOptimizer;

impl TransactionBatchOptimizer {
    /// Split instructions into optimally-sized transaction batches
    pub fn create_optimal_batches(
        instructions: Vec<Instruction>,
        signer_count: usize,
        config: TransactionOptimizationConfig,
    ) -> Vec<TransactionBatch> {
        let mut batches: Vec<TransactionBatch> = Vec::new();
        let mut current_batch = Vec::new();
        
        for instruction in instructions {
            // Check if adding this instruction would exceed limits
            let mut test_batch = current_batch.clone();
            test_batch.push(instruction.clone());
            
            if TransactionSizeEstimator::would_exceed_limit(&test_batch, signer_count, &config) ||
               test_batch.len() > config.max_instructions_per_tx {
                // Start new batch if current one isn't empty
                if !current_batch.is_empty() {
                    let estimated_size = TransactionSizeEstimator::estimate_transaction_size(&current_batch, signer_count);
                    batches.push(TransactionBatch {
                        instructions: current_batch,
                        estimated_size,
                        signer_count,
                    });
                    current_batch = Vec::new();
                }
            }
            
            current_batch.push(instruction);
        }
        
        // Add final batch if not empty
        if !current_batch.is_empty() {
            let estimated_size = TransactionSizeEstimator::estimate_transaction_size(&current_batch, signer_count);
            batches.push(TransactionBatch {
                instructions: current_batch,
                estimated_size,
                signer_count,
            });
        }
        
        batches
    }
    
    /// Optimize instruction ordering within batches for better packing
    pub fn optimize_instruction_ordering(
        instructions: &mut [Instruction],
        optimization_strategy: OrderingStrategy,
    ) {
        match optimization_strategy {
            OrderingStrategy::SizeAscending => {
                instructions.sort_by_key(TransactionSizeEstimator::estimate_instruction_size);
            },
            OrderingStrategy::SizeDescending => {
                instructions.sort_by_key(|inst| std::cmp::Reverse(TransactionSizeEstimator::estimate_instruction_size(inst)));
            },
            OrderingStrategy::AccountCountAscending => {
                instructions.sort_by_key(|inst| inst.accounts.len());
            },
            OrderingStrategy::DataSizeAscending => {
                instructions.sort_by_key(|inst| inst.data.len());
            },
            OrderingStrategy::Dependency => {
                // More complex dependency-based ordering would go here
                // For now, keep original order
            },
        }
    }
    
    /// Deduplicate accounts across instructions to reduce transaction size
    pub fn deduplicate_accounts(instructions: &mut [Instruction]) -> AccountDeduplicationResult {
        use std::collections::HashMap;
        
        let mut account_map: HashMap<Pubkey, usize> = HashMap::new();
        let mut deduplicated_accounts = Vec::new();
        let mut original_count = 0;
        
        for instruction in instructions.iter_mut() {
            original_count += instruction.accounts.len();
            
            for account_meta in instruction.accounts.iter_mut() {
                if let Some(&_index) = account_map.get(&account_meta.pubkey) {
                    // Account already exists, update index reference
                    // Note: This is a simplified approach - real implementation would need
                    // to handle account meta updates properly
                } else {
                    // New account, add to map and list
                    let index = deduplicated_accounts.len();
                    account_map.insert(account_meta.pubkey, index);
                    deduplicated_accounts.push(account_meta.clone());
                }
            }
        }
        
        AccountDeduplicationResult {
            original_account_count: original_count,
            deduplicated_account_count: deduplicated_accounts.len(),
            space_saved: (original_count - deduplicated_accounts.len()) * ACCOUNT_META_SIZE,
        }
    }
}

/// Instruction data compression utilities
pub struct InstructionDataCompressor;

impl InstructionDataCompressor {
    /// Compress instruction data using simple run-length encoding
    pub fn compress_instruction_data(data: &[u8]) -> Vec<u8> {
        if data.len() < 4 {
            return data.to_vec(); // Too small to compress effectively
        }
        
        let mut compressed = Vec::new();
        let mut i = 0;
        
        while i < data.len() {
            let current_byte = data[i];
            let mut count = 1;
            
            // Count consecutive identical bytes
            while i + count < data.len() && data[i + count] == current_byte && count < 255 {
                count += 1;
            }
            
            if count > 3 {
                // Use RLE for runs of 4 or more
                compressed.push(0xFF); // RLE marker
                compressed.push(count as u8);
                compressed.push(current_byte);
            } else {
                // Store bytes directly for short runs
                for _ in 0..count {
                    compressed.push(current_byte);
                }
            }
            
            i += count;
        }
        
        // Only return compressed version if it's actually smaller
        if compressed.len() < data.len() {
            compressed
        } else {
            data.to_vec()
        }
    }
    
    /// Decompress instruction data
    pub fn decompress_instruction_data(compressed_data: &[u8]) -> Vec<u8> {
        let mut decompressed = Vec::new();
        let mut i = 0;
        
        while i < compressed_data.len() {
            if compressed_data[i] == 0xFF && i + 2 < compressed_data.len() {
                // RLE sequence
                let count = compressed_data[i + 1] as usize;
                let byte_value = compressed_data[i + 2];
                
                for _ in 0..count {
                    decompressed.push(byte_value);
                }
                
                i += 3;
            } else {
                // Regular byte
                decompressed.push(compressed_data[i]);
                i += 1;
            }
        }
        
        decompressed
    }
}

/// Transaction batch representation
#[derive(Debug, Clone)]
pub struct TransactionBatch {
    pub instructions: Vec<Instruction>,
    pub estimated_size: usize,
    pub signer_count: usize,
}

impl TransactionBatch {
    /// Check if batch is within size limits
    pub fn is_within_limits(&self, config: &TransactionOptimizationConfig) -> bool {
        let target_size = (MAX_TRANSACTION_SIZE * config.target_size_percentage as usize) / 100;
        self.estimated_size <= target_size && 
        self.instructions.len() <= config.max_instructions_per_tx
    }
    
    /// Get efficiency ratio (instructions per byte)
    pub fn get_efficiency_ratio(&self) -> f64 {
        if self.estimated_size == 0 {
            0.0
        } else {
            self.instructions.len() as f64 / self.estimated_size as f64
        }
    }
}

/// Instruction ordering strategies for optimization
#[derive(Debug, Clone, Copy)]
pub enum OrderingStrategy {
    SizeAscending,
    SizeDescending,
    AccountCountAscending,
    DataSizeAscending,
    Dependency,
}

/// Account deduplication result
#[derive(Debug, Clone)]
pub struct AccountDeduplicationResult {
    pub original_account_count: usize,
    pub deduplicated_account_count: usize,
    pub space_saved: usize,
}

/// Transaction optimization analyzer
pub struct TransactionOptimizationAnalyzer;

impl TransactionOptimizationAnalyzer {
    /// Analyze transaction efficiency and suggest optimizations
    pub fn analyze_transaction_efficiency(
        batches: &[TransactionBatch],
        config: &TransactionOptimizationConfig,
    ) -> TransactionAnalysisReport {
        let total_instructions: usize = batches.iter().map(|b| b.instructions.len()).sum();
        let total_size: usize = batches.iter().map(|b| b.estimated_size).sum();
        let average_batch_size = if batches.is_empty() { 0 } else { total_size / batches.len() };
        
        let target_size = (MAX_TRANSACTION_SIZE * config.target_size_percentage as usize) / 100;
        let size_efficiency = if total_size == 0 { 0.0 } else {
            (total_instructions as f64) / (total_size as f64) * 1000.0 // Instructions per KB
        };
        
        let mut suggestions = Vec::new();
        
        // Analyze for optimization opportunities
        if average_batch_size < target_size / 2 {
            suggestions.push(OptimizationSuggestion::IncreaseBatchSize);
        }
        
        if batches.len() > 10 {
            suggestions.push(OptimizationSuggestion::ReduceBatchCount);
        }
        
        for (i, batch) in batches.iter().enumerate() {
            if batch.instructions.len() < 3 && batch.estimated_size < target_size / 3 {
                suggestions.push(OptimizationSuggestion::MergeBatch(i));
            }
        }
        
        TransactionAnalysisReport {
            total_batches: batches.len(),
            total_instructions,
            total_estimated_size: total_size,
            average_batch_size,
            size_efficiency,
            target_size,
            suggestions,
        }
    }
}

/// Transaction analysis report
#[derive(Debug, Clone)]
pub struct TransactionAnalysisReport {
    pub total_batches: usize,
    pub total_instructions: usize,
    pub total_estimated_size: usize,
    pub average_batch_size: usize,
    pub size_efficiency: f64,
    pub target_size: usize,
    pub suggestions: Vec<OptimizationSuggestion>,
}

/// Optimization suggestions
#[derive(Debug, Clone)]
pub enum OptimizationSuggestion {
    IncreaseBatchSize,
    ReduceBatchCount,
    MergeBatch(usize),
    CompressData,
    DeduplicateAccounts,
    ReorderInstructions,
}

/// Macro for easy transaction size estimation
#[macro_export]
macro_rules! estimate_tx_size {
    ($instructions:expr, $signers:expr) => {
        TransactionSizeEstimator::estimate_transaction_size($instructions, $signers)
    };
}

/// Macro for creating optimized transaction batches
#[macro_export]
macro_rules! optimize_batches {
    ($instructions:expr, $signers:expr) => {
        TransactionBatchOptimizer::create_optimal_batches(
            $instructions,
            $signers,
            TransactionOptimizationConfig::default()
        )
    };
    ($instructions:expr, $signers:expr, $config:expr) => {
        TransactionBatchOptimizer::create_optimal_batches($instructions, $signers, $config)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::pubkey::Pubkey;
    use solana_program::instruction::AccountMeta;
    
    #[test]
    fn test_transaction_size_estimation() {
        let instruction = Instruction {
            program_id: Pubkey::new_unique(),
            accounts: vec![
                AccountMeta::new(Pubkey::new_unique(), false),
                AccountMeta::new_readonly(Pubkey::new_unique(), false),
            ],
            data: vec![1, 2, 3, 4],
        };
        
        let size = TransactionSizeEstimator::estimate_instruction_size(&instruction);
        assert!(size > 0);
    }
    
    #[test]
    fn test_instruction_data_compression() {
        let data = vec![1, 1, 1, 1, 2, 2, 2, 3, 3, 3, 3, 3];
        let compressed = InstructionDataCompressor::compress_instruction_data(&data);
        let decompressed = InstructionDataCompressor::decompress_instruction_data(&compressed);
        
        assert_eq!(data, decompressed);
    }
    
    #[test]
    fn test_batch_optimization() {
        let instructions = vec![
            Instruction {
                program_id: Pubkey::new_unique(),
                accounts: vec![AccountMeta::new(Pubkey::new_unique(), false)],
                data: vec![1; 100],
            },
            Instruction {
                program_id: Pubkey::new_unique(),
                accounts: vec![AccountMeta::new(Pubkey::new_unique(), false)],
                data: vec![2; 100],
            },
        ];
        
        let batches = TransactionBatchOptimizer::create_optimal_batches(
            instructions,
            1,
            TransactionOptimizationConfig::default(),
        );
        
        assert!(!batches.is_empty());
    }
} 