// Compute unit metering and budget management for valence-kernel execution
//
// The valence-kernel processes complex batches of operations that can involve multiple
// account borrowings, CPI calls, and validation steps. To maintain Solana's strict
// compute budget requirements and provide predictable execution costs, this module
// provides comprehensive compute unit tracking and optimization analysis.
//
// BUDGET MANAGEMENT: Pre-execution cost estimation prevents transactions from failing
// due to compute budget exhaustion. The meter tracks actual usage against estimates
// to provide feedback for optimization and ensures operations complete within limits.
//
// PERFORMANCE OPTIMIZATION: Detailed categorized tracking (validation, CPI, account
// operations, etc.) helps identify performance bottlenecks in operation batches and
// enables targeted optimization of expensive execution paths.
//
// KERNEL INTEGRATION: The meter integrates with the batch execution engine to provide
// real-time budget monitoring and automatic execution throttling when approaching limits.

use anchor_lang::prelude::*;

// ================================
// Core Compute Unit Meter System
// ================================

/// Comprehensive compute budget management system
/// Provides cost estimation, runtime tracking, and optimization analysis
#[allow(clippy::module_inception)]
pub mod meter {
    use super::Result;
    #[cfg(debug_assertions)]
    use anchor_lang::prelude::msg;

    // ================================
    // Compute Unit Cost Constants
    // ================================

    /// Predefined compute unit costs for common operations
    /// These values are calibrated based on actual runtime measurements
    pub mod costs {
        // --- Base Operation Costs ---
        /// Cost to execute any instruction (fixed overhead)
        pub const INSTRUCTION_BASE: u64 = 150;
        /// Cost to read from a sysvar account
        pub const SYSVAR_ACCESS: u64 = 100;
        /// Cost to access an account (read account data)
        pub const ACCOUNT_ACCESS: u64 = 200;

        // --- State Management Costs ---
        /// Cost to deserialize and load state from account data
        pub const STATE_LOAD: u64 = 500;
        /// Cost to serialize and save state to account data
        pub const STATE_SAVE: u64 = 800;
        /// Cost to validate state integrity and constraints
        pub const STATE_VALIDATE: u64 = 300;

        // --- Guard Evaluation Costs ---
        /// Cost for inline guard evaluation (no external calls)
        pub const GUARD_INLINE: u64 = 500;
        /// Cost for cached guard result lookup
        pub const GUARD_CACHED_CHECK: u64 = 200;
        /// Cost for cross-program invocation to external guard
        pub const GUARD_CPI: u64 = 5000;

        // --- Cryptographic Operation Costs ---
        /// Cost to verify a cryptographic signature
        pub const VERIFY_SIGNATURE: u64 = 2000;
        /// Cost to compute or verify a hash
        pub const VERIFY_HASH: u64 = 500;

        // --- Data Serialization Costs ---
        /// Cost to serialize a Pubkey to bytes
        pub const SERIALIZE_PUBKEY: u64 = 50;
        /// Cost to deserialize bytes to Pubkey
        pub const DESERIALIZE_PUBKEY: u64 = 50;
        /// Cost to perform hash computation on data
        pub const HASH_OPERATION: u64 = 300;
    }

    // ================================
    // Runtime Budget Tracking
    // ================================

    /// Maximum number of categories to track
    const MAX_CATEGORIES: usize = 16;

    /// Category tracking entry
    #[derive(Clone, Copy, Default)]
    struct CategoryEntry {
        /// Category identifier (0 = empty)
        id: u8,
        /// Units used in this category
        units: u64,
    }

    /// Real-time compute unit budget tracker
    /// Monitors usage during instruction execution and provides warnings
    pub struct ComputeTracker {
        /// Total compute units allocated for this transaction
        pub budget: u64,

        /// Compute units consumed so far
        pub used: u64,

        /// Fixed-size array for category breakdown (avoids `HashMap` overhead)
        categories: [CategoryEntry; MAX_CATEGORIES],

        /// Number of active categories
        category_count: usize,

        /// Percentage threshold for usage warnings (0-100)
        pub warning_threshold: u8,
    }

    impl ComputeTracker {
        /// Default compute budget for standard transactions
        pub const DEFAULT_BUDGET: u64 = 200_000;
        /// Maximum compute budget allowed per transaction
        pub const MAX_BUDGET: u64 = 1_400_000;

        /// Create a new budget tracker with specified allocation
        #[must_use]
        pub fn new(budget: u64) -> Self {
            Self {
                budget: budget.min(Self::MAX_BUDGET), // Enforce Solana limits
                used: 0,
                categories: [CategoryEntry::default(); MAX_CATEGORIES],
                category_count: 0,
                warning_threshold: 80, // Warn at 80% usage
            }
        }

        /// Record compute unit usage for a specific operation category
        /// 
        /// # Errors
        /// Returns error if budget would be exceeded
        pub fn track(&mut self, category: &str, units: u64) -> Result<()> {
            // Update total usage with overflow protection
            self.used = self.used.saturating_add(units);

            // Check for budget violation
            if self.used > self.budget {
                return Err(crate::errors::KernelError::ComputeBudgetExceeded.into());
            }

            // Update category breakdown using fixed array
            let category_id = Self::hash_category_to_id(category);
            
            // Find existing category or add new one
            let mut found = false;
            for i in 0..self.category_count {
                if self.categories[i].id == category_id {
                    self.categories[i].units += units;
                    found = true;
                    break;
                }
            }
            
            // Add new category if not found and there's space
            if !found && self.category_count < MAX_CATEGORIES {
                self.categories[self.category_count] = CategoryEntry {
                    id: category_id,
                    units,
                };
                self.category_count += 1;
            }

            Ok(())
        }

        /// Convert category string to numeric ID for efficient storage
        /// Uses simple hash to convert string to u8
        #[allow(clippy::cast_possible_truncation)]
        fn hash_category_to_id(category: &str) -> u8 {
            // Simple hash: sum of bytes modulo 255, ensuring non-zero
            let sum: u32 = category.bytes().map(u32::from).sum();
            ((sum % 255) + 1) as u8
        }

    /// Calculate remaining compute units in budget
    #[must_use]
    pub const fn remaining(&self) -> u64 {
        self.budget.saturating_sub(self.used)
    }

    /// Calculate current usage as percentage of total budget
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn usage_percentage(&self) -> u8 {
        ((self.used * 100) / self.budget) as u8
    }

    /// Check if usage exceeds the warning threshold
    #[must_use]
    pub const fn is_over_threshold(&self) -> bool {
        self.usage_percentage() >= self.warning_threshold
    }

    /// Check if budget has been completely exhausted
    #[must_use]
    pub const fn is_budget_exceeded(&self) -> bool {
        self.used >= self.budget
    }

        /// Output detailed usage breakdown for debugging
        #[cfg(debug_assertions)]
        pub fn log_breakdown(&self) {
            msg!("Compute Budget: {}/{} CU ({}%)", self.used, self.budget, self.usage_percentage());
            if self.category_count > 0 {
                let mut cats = self.categories[..self.category_count].to_vec();
                cats.sort_by(|a, b| b.units.cmp(&a.units));
                for cat in cats {
                    msg!("  Cat {}: {} CU", cat.id, cat.units);
                }
            }
        }
        
        #[cfg(not(debug_assertions))]
        pub fn log_breakdown(&self) {}
    }
} // End of meter module

// ================================
// Operation Cost Estimation
// ================================

/// Cost estimation for specific operations
/// Provides structured way to calculate compute unit requirements
#[derive(Debug, Clone)]
pub struct OperationCost {
    /// Fixed-size name buffer for debugging
    pub name: [u8; 32],
    /// Actual name length
    pub name_len: u8,
    /// Base compute cost regardless of accounts
    pub base_cost: u64,
    /// Additional cost per account involved
    pub per_account_cost: u64,
    /// Whether this operation involves CPI calls
    pub has_cpi: bool,
}

impl OperationCost {
    /// Estimate total compute units for this operation with given account count
    #[must_use]
    pub const fn estimate(&self, account_count: usize) -> u64 {
        let mut total = self.base_cost;
        
        // Add per-account costs
        total += self.per_account_cost * account_count as u64;
        
        // Add CPI overhead if applicable
        if self.has_cpi {
            total += costs::GUARD_CPI;
        }
        
        total
    }
}

// ================================
// Public API Re-exports
// ================================

// Re-export commonly used items for easier access
pub use meter::{costs, ComputeTracker};

// ================================
// Convenience Macros
// ================================

/// Macro for tracking compute units with automatic error handling
#[macro_export]
macro_rules! track_compute {
    // Simple usage: track_compute!(tracker, "category", 100)
    ($tracker:expr, $category:expr, $units:expr) => {
        $tracker.track($category, $units)?
    };

    // With operation: track_compute!(tracker, "category", 100, { some_operation() })
    ($tracker:expr, $category:expr, $units:expr, $op:expr) => {{
        $tracker.track($category, $units)?;
        $op
    }};
}

/// Macro for logging compute usage during development (debug only)
#[macro_export]
macro_rules! with_compute_tracking {
    ($category:expr, $units:expr, $op:expr) => {{
        #[cfg(debug_assertions)]
        msg!("CU: {} +{}", $category, $units);
        $op
    }};
}
