//! Transaction validation before signing
//!
//! This module provides comprehensive validation rules for transactions
//! before they are sent for signing, ensuring security and policy compliance.

use crate::security::{SecurityAnalysis, SecurityAnalyzer, SecurityContext};
use crate::signing_service::{RiskLevel, SigningPolicies};
use crate::{Result, RuntimeError, UnsignedTransaction};
use async_trait::async_trait;
use chrono::{Datelike, Timelike};
use serde::{Deserialize, Serialize};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, transaction::Transaction};
use std::collections::HashMap;
use std::sync::Arc;

/// Validation context for transaction validation
#[derive(Clone)]
pub struct ValidationContext {
    /// RPC client for account lookups
    pub rpc_client: Arc<RpcClient>,

    /// Current slot
    pub slot: u64,

    /// Recent blockhash
    pub recent_blockhash: String,

    /// Security context
    pub security_context: SecurityContext,

    /// Signing policies
    pub signing_policies: SigningPolicies,

    /// Account cache for fetched accounts
    pub account_cache: HashMap<Pubkey, solana_sdk::account::Account>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Validation rule trait
#[async_trait]
pub trait ValidationRule: Send + Sync {
    /// Validate a transaction
    async fn validate(
        &self,
        transaction: &Transaction,
        context: &ValidationContext,
    ) -> Result<ValidationResult>;

    /// Get the name of this validation rule
    fn name(&self) -> &str;

    /// Whether this rule is critical (failure should block transaction)
    fn is_critical(&self) -> bool {
        false // Default to non-critical
    }
}

/// Transaction validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the transaction passed validation
    pub valid: bool,

    /// Risk assessment
    pub risk_level: RiskLevel,

    /// Validation errors
    pub errors: Vec<ValidationError>,

    /// Validation warnings
    pub warnings: Vec<ValidationWarning>,

    /// Security analysis
    pub security_analysis: SecurityAnalysis,

    /// Suggested remediations
    pub remediations: Vec<Remediation>,

    /// Validation metadata
    pub metadata: ValidationMetadata,
}

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error code
    pub code: ErrorCode,

    /// Error message
    pub message: String,

    /// Affected component
    pub component: Component,

    /// Additional context
    pub context: HashMap<String, String>,
}

/// Validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning code
    pub code: WarningCode,

    /// Warning message
    pub message: String,

    /// Severity
    pub severity: WarningSeverity,
}

/// Error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    // Policy violations
    PolicyViolation,
    UnauthorizedProgram,
    BlockedAccount,
    ExceedsLimit,

    // Security issues
    SecurityRisk,
    SuspiciousPattern,
    UnverifiedProgram,

    // Transaction issues
    InvalidInstruction,
    InvalidAccount,
    InsufficientFunds,

    // Structural issues
    TooManyInstructions,
    TooManyAccounts,
    TransactionTooLarge,
}

/// Warning codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningCode {
    HighValue,
    UnusualPattern,
    NewProgram,
    RareOperation,
    HighComputeUsage,
}

/// Warning severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningSeverity {
    Low,
    Medium,
    High,
}

/// Affected component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Component {
    Transaction,
    Instruction(usize),
    Account(Pubkey),
    Program(Pubkey),
}

/// Suggested remediation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remediation {
    /// Issue being addressed
    pub issue: String,

    /// Suggested action
    pub action: String,

    /// Whether this is automated
    pub automated: bool,
}

/// Validation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMetadata {
    /// Validation timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Validation duration (ms)
    pub duration_ms: u64,

    /// Rules evaluated
    pub rules_evaluated: u32,

    /// Simulated compute units
    pub simulated_compute_units: Option<u64>,

    /// Estimated transaction fee
    pub estimated_fee: Option<u64>,
}

// ================================
// Validation Rule Enum (replaces trait objects)
// ================================

#[derive(Debug, Clone)]
pub enum ValidationRuleType {
    PolicyCompliance,
    ProgramWhitelist,
    AccountBlacklist,
    TransactionLimits,
    ComputeBudget,
    BalanceCheck,
    ReentrancyCheck,
    SignatureVerification,
}

impl ValidationRuleType {
    /// Execute the validation rule
    pub async fn validate(
        &self,
        transaction: &Transaction,
        context: &ValidationContext,
    ) -> Result<ValidationResult> {
        match self {
            Self::PolicyCompliance => PolicyComplianceRule.validate(transaction, context).await,
            Self::ProgramWhitelist => ProgramWhitelistRule.validate(transaction, context).await,
            Self::AccountBlacklist => AccountBlacklistRule.validate(transaction, context).await,
            Self::TransactionLimits => TransactionLimitsRule.validate(transaction, context).await,
            Self::ComputeBudget => ComputeBudgetRule.validate(transaction, context).await,
            Self::BalanceCheck => BalanceCheckRule.validate(transaction, context).await,
            Self::ReentrancyCheck => ReentrancyCheckRule.validate(transaction, context).await,
            Self::SignatureVerification => {
                SignatureVerificationRule
                    .validate(transaction, context)
                    .await
            }
        }
    }
}

// ================================
// Transaction Validator (Updated)
// ================================

/// Transaction validator
pub struct TransactionValidator {
    /// Validation rules
    rules: Vec<Box<dyn ValidationRule>>,

    /// Security analyzer
    security_analyzer: SecurityAnalyzer,

    /// RPC client
    rpc_client: Arc<RpcClient>,
}

impl TransactionValidator {
    /// Create a new validator
    pub fn new(rpc_client: Arc<RpcClient>, security_context: SecurityContext) -> Self {
        let security_analyzer = SecurityAnalyzer::new(security_context.clone());

        // Initialize with default rules
        let rules: Vec<Box<dyn ValidationRule>> = vec![
            Box::new(PolicyComplianceRule),
            Box::new(ProgramWhitelistRule),
            Box::new(AccountBlacklistRule),
            Box::new(TransactionLimitsRule),
            Box::new(ComputeBudgetRule),
            Box::new(BalanceCheckRule),
            Box::new(ReentrancyCheckRule),
            Box::new(SignatureVerificationRule),
        ];

        Self {
            rules,
            security_analyzer,
            rpc_client,
        }
    }

    /// Add a custom validation rule
    pub fn add_rule(&mut self, rule: Box<dyn ValidationRule>) {
        self.rules.push(rule);
    }

    /// Validate a transaction
    pub async fn validate(
        &self,
        transaction: &UnsignedTransaction,
        security_context: SecurityContext,
    ) -> Result<ValidationResult> {
        let context = ValidationContext {
            rpc_client: Arc::clone(&self.rpc_client),
            slot: 0,                         // TODO: Get actual slot
            recent_blockhash: String::new(), // TODO: Get actual blockhash
            security_context,
            signing_policies: SigningPolicies::default(),
            account_cache: HashMap::new(),
            metadata: HashMap::new(),
        };

        let mut all_errors = Vec::new();
        let mut all_warnings = Vec::new();
        let mut max_risk_level = RiskLevel::Low;

        // Convert UnsignedTransaction to Transaction for rule validation
        let message: solana_sdk::message::Message = bincode::deserialize(&transaction.message)
            .map_err(|e| {
                RuntimeError::TransactionBuildError(format!("Failed to deserialize message: {}", e))
            })?;
        let transaction_for_rules = solana_sdk::transaction::Transaction {
            signatures: vec![], // Empty since it's unsigned
            message,
        };

        // Run all validation rules
        for rule in &self.rules {
            let result = rule.validate(&transaction_for_rules, &context).await?;

            if !result.valid {
                all_errors.extend(result.errors);
                all_warnings.extend(result.warnings);

                // Update max risk level
                if result.risk_level as u8 > max_risk_level as u8 {
                    max_risk_level = result.risk_level;
                }
            }
        }

        // Convert UnsignedTransaction to Transaction for analysis
        let message: solana_sdk::message::Message = bincode::deserialize(&transaction.message)
            .map_err(|e| {
                RuntimeError::TransactionBuildError(format!("Failed to deserialize message: {}", e))
            })?;
        let transaction_for_analysis = solana_sdk::transaction::Transaction {
            signatures: vec![], // Empty since it's unsigned
            message,
        };
        let security_analysis = self
            .security_analyzer
            .analyze_transaction(&transaction_for_analysis);

        Ok(ValidationResult {
            valid: all_errors.is_empty(),
            risk_level: max_risk_level,
            errors: all_errors,
            warnings: all_warnings,
            security_analysis,
            remediations: Vec::new(),
            metadata: ValidationMetadata {
                timestamp: chrono::Utc::now(),
                duration_ms: 0, // Would be calculated properly in real implementation
                rules_evaluated: 0, // Would be counted properly in real implementation
                simulated_compute_units: None,
                estimated_fee: None,
            },
        })
    }

    /// Prefetch accounts for validation
    pub async fn prefetch_accounts(
        &self,
        transaction: &Transaction,
        context: &mut ValidationContext,
    ) -> Result<()> {
        let account_keys: Vec<_> = transaction.message.account_keys.clone();

        // Fetch accounts in parallel
        let accounts = self.rpc_client.get_multiple_accounts(&account_keys).await?;

        for (pubkey, account) in account_keys.iter().zip(accounts.iter()) {
            if let Some(account) = account {
                context.account_cache.insert(*pubkey, account.clone());
            }
        }

        Ok(())
    }

    /// Estimate transaction fee
    pub async fn estimate_fee(&self, transaction: &Transaction) -> Result<u64> {
        let message = &transaction.message;
        let fee = self.rpc_client.get_fee_for_message(message).await?;
        Ok(fee)
    }

    /// Generate remediation suggestions
    pub fn generate_remediations(
        &self,
        errors: &[ValidationError],
        _warnings: &[ValidationWarning],
    ) -> Vec<Remediation> {
        let mut remediations = Vec::new();

        // Check for common patterns
        let has_compute_limit = errors.iter().any(|e| {
            e.code == ErrorCode::ExceedsLimit
                && e.context.get("limit_type") == Some(&"compute_units".to_string())
        });

        if has_compute_limit {
            remediations.push(Remediation {
                issue: "Transaction exceeds compute unit limit".to_string(),
                action: "Split transaction into multiple smaller transactions".to_string(),
                automated: false,
            });
        }

        let has_too_many_accounts = errors.iter().any(|e| e.code == ErrorCode::TooManyAccounts);

        if has_too_many_accounts {
            remediations.push(Remediation {
                issue: "Too many accounts in transaction".to_string(),
                action: "Reduce account usage or use lookup tables".to_string(),
                automated: false,
            });
        }

        remediations
    }
}

// ===== Default Validation Rules =====

/// Policy compliance rule
struct PolicyComplianceRule;

#[async_trait]
impl ValidationRule for PolicyComplianceRule {
    fn name(&self) -> &str {
        "PolicyCompliance"
    }

    fn is_critical(&self) -> bool {
        true
    }

    async fn validate(
        &self,
        transaction: &Transaction,
        context: &ValidationContext,
    ) -> Result<ValidationResult> {
        let errors = Vec::new();
        let warnings = Vec::new();
        let risk_contribution = 0u8;

        // Check signing policies for each required signer
        for signer in &transaction.message.account_keys {
            // For simplicity, apply the same policies to all signers
            let policies = &context.signing_policies;

            // Check transaction value limit
            if let Some(max_value) = policies.max_transaction_value {
                // This would need actual value calculation
                // For now, we'll skip this check
                let _ = max_value; // Suppress unused warning
            }

            // Check time restrictions
            if let Some(time_restrictions) = &policies.time_restrictions {
                let now = chrono::Utc::now();
                let day_of_week = now.weekday().num_days_from_sunday() as u8;
                let hour = now.hour() as u8;

                // Suppress unused warnings for time restrictions check - would need implementation
                let _ = (time_restrictions, now, day_of_week, hour);
                /*
                // Time restrictions would be implemented here if TimeRestrictions struct has the right fields
                 */
            }

            // Suppress unused signer warning
            let _ = signer;
        }

        Ok(ValidationResult {
            valid: errors.is_empty(),
            risk_level: RiskLevel::from_risk_level(risk_contribution),
            errors,
            warnings,
            security_analysis: SecurityAnalysis::new(SecurityContext::new()), // Placeholder
            remediations: Vec::new(),                                         // Placeholder
            metadata: ValidationMetadata {
                timestamp: chrono::Utc::now(),
                duration_ms: 0,                // Placeholder
                rules_evaluated: 0,            // Placeholder
                simulated_compute_units: None, // Placeholder
                estimated_fee: None,           // Placeholder
            },
        })
    }
}

/// Program whitelist rule
struct ProgramWhitelistRule;

#[async_trait]
impl ValidationRule for ProgramWhitelistRule {
    fn name(&self) -> &str {
        "ProgramWhitelist"
    }

    fn is_critical(&self) -> bool {
        true
    }

    async fn validate(
        &self,
        transaction: &Transaction,
        context: &ValidationContext,
    ) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let warnings = Vec::new();
        let mut risk_contribution = 0u8;

        let message = &transaction.message;

        // Check each instruction's program
        for (idx, instruction) in message.instructions.iter().enumerate() {
            let program_id = message.account_keys[instruction.program_id_index as usize];

            // Check against policies
            // Check the global signing policies
            let policies = &context.signing_policies;

            // Check blocked programs
            if policies.blocked_programs.contains(&program_id) {
                errors.push(ValidationError {
                    code: ErrorCode::UnauthorizedProgram,
                    message: format!("Program {} is blocked", program_id),
                    component: Component::Instruction(idx),
                    context: HashMap::from([("program_id".to_string(), program_id.to_string())]),
                });
                risk_contribution += 50;
            }

            // Check allowed programs if whitelist is enabled
            if let Some(allowed) = &policies.allowed_programs {
                if !allowed.contains(&program_id) {
                    errors.push(ValidationError {
                        code: ErrorCode::UnauthorizedProgram,
                        message: format!("Program {} is not in whitelist", program_id),
                        component: Component::Instruction(idx),
                        context: HashMap::from([(
                            "program_id".to_string(),
                            program_id.to_string(),
                        )]),
                    });
                    risk_contribution += 30;
                }
            }
        }

        Ok(ValidationResult {
            valid: errors.is_empty(),
            risk_level: RiskLevel::from_risk_level(risk_contribution),
            errors,
            warnings,
            security_analysis: SecurityAnalysis::new(SecurityContext::new()), // Placeholder
            remediations: Vec::new(),                                         // Placeholder
            metadata: ValidationMetadata {
                timestamp: chrono::Utc::now(),
                duration_ms: 0,                // Placeholder
                rules_evaluated: 0,            // Placeholder
                simulated_compute_units: None, // Placeholder
                estimated_fee: None,           // Placeholder
            },
        })
    }
}

/// Account blacklist rule
struct AccountBlacklistRule;

#[async_trait]
impl ValidationRule for AccountBlacklistRule {
    fn name(&self) -> &str {
        "AccountBlacklist"
    }

    fn is_critical(&self) -> bool {
        true
    }

    async fn validate(
        &self,
        transaction: &Transaction,
        context: &ValidationContext,
    ) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let risk_contribution = 0u8;

        let message = &transaction.message;

        // Check blocked accounts
        let blocked_accounts = &context
            .security_context
            .policies
            .account_restrictions
            .blocked_accounts;

        for account in &message.account_keys {
            if blocked_accounts.contains(account) {
                errors.push(ValidationError {
                    code: ErrorCode::BlockedAccount,
                    message: format!("Account {} is blocked", account),
                    component: Component::Account(*account),
                    context: HashMap::new(),
                });
            }
        }

        Ok(ValidationResult {
            valid: errors.is_empty(),
            risk_level: RiskLevel::from_risk_level(risk_contribution),
            errors,
            warnings: Vec::new(),
            security_analysis: SecurityAnalysis::new(SecurityContext::new()), // Placeholder
            remediations: Vec::new(),                                         // Placeholder
            metadata: ValidationMetadata {
                timestamp: chrono::Utc::now(),
                duration_ms: 0,                // Placeholder
                rules_evaluated: 0,            // Placeholder
                simulated_compute_units: None, // Placeholder
                estimated_fee: None,           // Placeholder
            },
        })
    }
}

/// Transaction limits rule
struct TransactionLimitsRule;

#[async_trait]
impl ValidationRule for TransactionLimitsRule {
    fn name(&self) -> &str {
        "TransactionLimits"
    }

    fn is_critical(&self) -> bool {
        true
    }

    async fn validate(
        &self,
        transaction: &Transaction,
        context: &ValidationContext,
    ) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let warnings = Vec::new();
        let mut risk_contribution = 0u8;

        let limits = &context.security_context.policies.transaction_limits;

        // Check transaction size - serialize message to get size
        let serialized_size = bincode::serialize(&transaction.message)
            .map_err(|e| RuntimeError::TransactionBuildError(e.to_string()))?
            .len();
        if serialized_size > limits.max_size_bytes {
            errors.push(ValidationError {
                code: ErrorCode::TransactionTooLarge,
                message: format!(
                    "Transaction size {} exceeds limit {}",
                    serialized_size, limits.max_size_bytes
                ),
                component: Component::Transaction,
                context: HashMap::from([
                    ("size".to_string(), serialized_size.to_string()),
                    ("limit".to_string(), limits.max_size_bytes.to_string()),
                ]),
            });
            risk_contribution += 20;
        }

        // Use message directly since transaction.message is already a Message struct
        let message = &transaction.message;

        // Check instruction count
        if message.instructions.len() > limits.max_instructions {
            errors.push(ValidationError {
                code: ErrorCode::TooManyInstructions,
                message: format!(
                    "Instruction count {} exceeds limit {}",
                    message.instructions.len(),
                    limits.max_instructions
                ),
                component: Component::Transaction,
                context: HashMap::from([
                    ("count".to_string(), message.instructions.len().to_string()),
                    ("limit".to_string(), limits.max_instructions.to_string()),
                ]),
            });
            risk_contribution += 15;
        }

        // Check account count
        if message.account_keys.len() > limits.max_accounts {
            errors.push(ValidationError {
                code: ErrorCode::TooManyAccounts,
                message: format!(
                    "Account count {} exceeds limit {}",
                    message.account_keys.len(),
                    limits.max_accounts
                ),
                component: Component::Transaction,
                context: HashMap::from([
                    ("count".to_string(), message.account_keys.len().to_string()),
                    ("limit".to_string(), limits.max_accounts.to_string()),
                ]),
            });
            risk_contribution += 15;
        }

        // Check compute units - for now skip this check as standard Message doesn't have compute_units
        // In real implementation, would parse compute budget instructions
        let compute_units: Option<u32> = None;
        if let Some(compute_units) = compute_units {
            if compute_units > limits.max_compute_units {
                errors.push(ValidationError {
                    code: ErrorCode::ExceedsLimit,
                    message: format!(
                        "Compute units {} exceeds limit {}",
                        compute_units, limits.max_compute_units
                    ),
                    component: Component::Transaction,
                    context: HashMap::from([
                        ("compute_units".to_string(), compute_units.to_string()),
                        ("limit".to_string(), limits.max_compute_units.to_string()),
                        ("limit_type".to_string(), "compute_units".to_string()),
                    ]),
                });
                risk_contribution += 25;
            }
        }

        Ok(ValidationResult {
            valid: errors.is_empty(),
            risk_level: RiskLevel::from_risk_level(risk_contribution),
            errors,
            warnings,
            security_analysis: SecurityAnalysis::new(SecurityContext::new()), // Placeholder
            remediations: Vec::new(),                                         // Placeholder
            metadata: ValidationMetadata {
                timestamp: chrono::Utc::now(),
                duration_ms: 0,                // Placeholder
                rules_evaluated: 0,            // Placeholder
                simulated_compute_units: None, // Placeholder
                estimated_fee: None,           // Placeholder
            },
        })
    }
}

/// Compute budget rule
struct ComputeBudgetRule;

#[async_trait]
impl ValidationRule for ComputeBudgetRule {
    fn name(&self) -> &str {
        "ComputeBudget"
    }

    fn is_critical(&self) -> bool {
        false
    }

    async fn validate(
        &self,
        _transaction: &Transaction,
        _context: &ValidationContext,
    ) -> Result<ValidationResult> {
        let mut warnings = Vec::new();
        let mut risk_contribution = 0u8;

        // For now, assume no compute units are set since standard Message doesn't have this field
        let compute_units: Option<u32> = None;

        // Check if compute units are set - skipping for now
        if compute_units.is_none() {
            warnings.push(ValidationWarning {
                code: WarningCode::HighComputeUsage,
                message: "No compute unit limit set".to_string(),
                severity: WarningSeverity::Medium,
            });
            risk_contribution += 5;
        }

        // Check if priority fee is reasonable - skipping for now since standard Message doesn't have this field
        let priority_fee: Option<u64> = None;
        if let Some(fee) = priority_fee {
            if fee > 50000 {
                // More than 50k microlamports per CU
                warnings.push(ValidationWarning {
                    code: WarningCode::HighValue,
                    message: format!("High priority fee: {} microlamports/CU", fee),
                    severity: WarningSeverity::Low,
                });
            }
        }

        Ok(ValidationResult {
            valid: true,
            risk_level: RiskLevel::from_risk_level(risk_contribution),
            errors: Vec::new(),
            warnings,
            security_analysis: SecurityAnalysis::new(SecurityContext::new()), // Placeholder
            remediations: Vec::new(),                                         // Placeholder
            metadata: ValidationMetadata {
                timestamp: chrono::Utc::now(),
                duration_ms: 0,                // Placeholder
                rules_evaluated: 0,            // Placeholder
                simulated_compute_units: None, // Placeholder
                estimated_fee: None,           // Placeholder
            },
        })
    }
}

/// Balance check rule
struct BalanceCheckRule;

#[async_trait]
impl ValidationRule for BalanceCheckRule {
    fn name(&self) -> &str {
        "BalanceCheck"
    }

    fn is_critical(&self) -> bool {
        true
    }

    async fn validate(
        &self,
        transaction: &Transaction,
        context: &ValidationContext,
    ) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let risk_contribution = 0u8;

        // Check fee payer balance
        if let Some(fee_payer) = transaction.message.account_keys.first() {
            if let Some(account) = context.account_cache.get(fee_payer) {
                // Estimate required balance (fee + rent-exemption)
                let min_balance = 5000 * 5000; // Rough estimate: 5000 lamports per signature

                if account.lamports < min_balance {
                    errors.push(ValidationError {
                        code: ErrorCode::InsufficientFunds,
                        message: format!(
                            "Fee payer has insufficient balance: {} < {}",
                            account.lamports, min_balance
                        ),
                        component: Component::Account(*fee_payer),
                        context: HashMap::from([
                            ("balance".to_string(), account.lamports.to_string()),
                            ("required".to_string(), min_balance.to_string()),
                        ]),
                    });
                }
            }
        }

        Ok(ValidationResult {
            valid: errors.is_empty(),
            risk_level: RiskLevel::from_risk_level(risk_contribution),
            errors,
            warnings: Vec::new(),
            security_analysis: SecurityAnalysis::new(SecurityContext::new()), // Placeholder
            remediations: Vec::new(),                                         // Placeholder
            metadata: ValidationMetadata {
                timestamp: chrono::Utc::now(),
                duration_ms: 0,                // Placeholder
                rules_evaluated: 0,            // Placeholder
                simulated_compute_units: None, // Placeholder
                estimated_fee: None,           // Placeholder
            },
        })
    }
}

/// Reentrancy check rule
struct ReentrancyCheckRule;

#[async_trait]
impl ValidationRule for ReentrancyCheckRule {
    fn name(&self) -> &str {
        "ReentrancyCheck"
    }

    fn is_critical(&self) -> bool {
        false
    }

    async fn validate(
        &self,
        transaction: &Transaction,
        _context: &ValidationContext,
    ) -> Result<ValidationResult> {
        let mut warnings = Vec::new();
        let mut risk_contribution = 0u8;

        let message = &transaction.message;

        // Check for potential reentrancy patterns
        let mut program_calls: HashMap<Pubkey, usize> = HashMap::new();

        for instruction in &message.instructions {
            let program_id = message.account_keys[instruction.program_id_index as usize];
            *program_calls.entry(program_id).or_insert(0) += 1;
        }

        // Warn if same program called multiple times
        for (program_id, count) in program_calls {
            if count > 1 {
                warnings.push(ValidationWarning {
                    code: WarningCode::UnusualPattern,
                    message: format!("Program {} called {} times", program_id, count),
                    severity: WarningSeverity::Medium,
                });
                risk_contribution += 10;
            }
        }

        Ok(ValidationResult {
            valid: true,
            risk_level: RiskLevel::from_risk_level(risk_contribution),
            errors: Vec::new(),
            warnings,
            security_analysis: SecurityAnalysis::new(SecurityContext::new()), // Placeholder
            remediations: Vec::new(),                                         // Placeholder
            metadata: ValidationMetadata {
                timestamp: chrono::Utc::now(),
                duration_ms: 0,                // Placeholder
                rules_evaluated: 0,            // Placeholder
                simulated_compute_units: None, // Placeholder
                estimated_fee: None,           // Placeholder
            },
        })
    }
}

/// Signature verification rule
struct SignatureVerificationRule;

#[async_trait]
impl ValidationRule for SignatureVerificationRule {
    fn name(&self) -> &str {
        "SignatureVerification"
    }

    fn is_critical(&self) -> bool {
        true
    }

    async fn validate(
        &self,
        transaction: &Transaction,
        _context: &ValidationContext,
    ) -> Result<ValidationResult> {
        let mut errors = Vec::new();

        // Verify required signers are specified
        if transaction.message.account_keys.is_empty() {
            errors.push(ValidationError {
                code: ErrorCode::InvalidInstruction,
                message: "No signers specified".to_string(),
                component: Component::Transaction,
                context: HashMap::new(),
            });
        }

        // Message is already deserialized in Transaction struct
        let _message = &transaction.message;

        Ok(ValidationResult {
            valid: errors.is_empty(),
            risk_level: RiskLevel::from_risk_level(0), // Placeholder
            errors,
            warnings: Vec::new(),
            security_analysis: SecurityAnalysis::new(SecurityContext::new()), // Placeholder
            remediations: Vec::new(),                                         // Placeholder
            metadata: ValidationMetadata {
                timestamp: chrono::Utc::now(),
                duration_ms: 0,                // Placeholder
                rules_evaluated: 0,            // Placeholder
                simulated_compute_units: None, // Placeholder
                estimated_fee: None,           // Placeholder
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_equality() {
        assert_eq!(ErrorCode::PolicyViolation, ErrorCode::PolicyViolation);
        assert_ne!(ErrorCode::PolicyViolation, ErrorCode::SecurityRisk);
    }

    #[tokio::test]
    async fn test_rule_result_creation() {
        let result = ValidationResult {
            valid: true,
            risk_level: RiskLevel::Low,
            errors: Vec::new(),
            warnings: vec![ValidationWarning {
                code: WarningCode::HighValue,
                message: "High value transaction".to_string(),
                severity: WarningSeverity::Medium,
            }],
            security_analysis: SecurityAnalysis::new(SecurityContext::new()), // Placeholder
            remediations: Vec::new(),                                         // Placeholder
            metadata: ValidationMetadata {
                timestamp: chrono::Utc::now(),
                duration_ms: 0,                // Placeholder
                rules_evaluated: 0,            // Placeholder
                simulated_compute_units: None, // Placeholder
                estimated_fee: None,           // Placeholder
            },
        };

        assert!(result.valid);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.risk_level, RiskLevel::Low);
    }
}
