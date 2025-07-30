//! Transaction validation before signing

use crate::security::{SecurityAnalysis, SecurityAnalyzer, SecurityContext};
use crate::security::signing::{RiskLevel, SigningPolicies};
use crate::{Result, RuntimeError, UnsignedTransaction};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, transaction::Transaction};
use std::collections::HashMap;
use std::sync::Arc;

/// Validation context for transaction validation
#[derive(Clone)]
pub struct ValidationContext {
    pub rpc_client: Arc<RpcClient>,
    pub security_context: SecurityContext,
    pub signing_policies: SigningPolicies,
}

/// Transaction validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub risk_level: RiskLevel,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub security_analysis: SecurityAnalysis,
}

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub code: ErrorCode,
    pub message: String,
    pub component: Component,
}

/// Validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub code: WarningCode,
    pub message: String,
    pub severity: WarningSeverity,
}

/// Error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    PolicyViolation,
    UnauthorizedProgram,
    BlockedAccount,
    ExceedsLimit,
    SecurityRisk,
    InvalidInstruction,
    InsufficientFunds,
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

/// Validation rule trait
#[async_trait]
pub trait ValidationRule: Send + Sync {
    async fn validate(&self, transaction: &Transaction, context: &ValidationContext) -> Result<ValidationResult>;
    fn name(&self) -> &str;
    fn is_critical(&self) -> bool { false }
}

/// Transaction validator
pub struct TransactionValidator {
    rules: Vec<Box<dyn ValidationRule>>,
    security_analyzer: SecurityAnalyzer,
    rpc_client: Arc<RpcClient>,
}

impl TransactionValidator {
    /// Create a new validator
    pub fn new(rpc_client: Arc<RpcClient>, security_context: SecurityContext) -> Self {
        let security_analyzer = SecurityAnalyzer::new(security_context.clone());
        let rules: Vec<Box<dyn ValidationRule>> = vec![
            Box::new(BasicValidationRule),
            Box::new(SecurityValidationRule),
        ];

        Self { rules, security_analyzer, rpc_client }
    }

    /// Validate a transaction
    pub async fn validate(&self, transaction: &UnsignedTransaction, security_context: SecurityContext) -> Result<ValidationResult> {
        let context = ValidationContext {
            rpc_client: Arc::clone(&self.rpc_client),
            security_context,
            signing_policies: SigningPolicies::default(),
        };

        let mut all_errors = Vec::new();
        let mut all_warnings = Vec::new();
        let mut max_risk_level = RiskLevel::Low;

        // Convert UnsignedTransaction to Transaction
        let message: solana_sdk::message::Message = bincode::deserialize(&transaction.message)
            .map_err(|e| RuntimeError::TransactionBuildError(format!("Failed to deserialize: {}", e)))?;
        let tx = Transaction { signatures: vec![], message };

        // Run validation rules
        for rule in &self.rules {
            let result = rule.validate(&tx, &context).await?;
            if !result.valid {
                all_errors.extend(result.errors);
                all_warnings.extend(result.warnings);
                if result.risk_level as u8 > max_risk_level as u8 {
                    max_risk_level = result.risk_level;
                }
            }
        }

        let security_analysis = self.security_analyzer.analyze_transaction(&tx);

        Ok(ValidationResult {
            valid: all_errors.is_empty(),
            risk_level: max_risk_level,
            errors: all_errors,
            warnings: all_warnings,
            security_analysis,
        })
    }
}

/// Basic validation rule
struct BasicValidationRule;

#[async_trait]
impl ValidationRule for BasicValidationRule {
    fn name(&self) -> &str { "BasicValidation" }
    fn is_critical(&self) -> bool { true }

    async fn validate(&self, transaction: &Transaction, context: &ValidationContext) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let warnings = Vec::new();
        let mut risk_contribution = 0u8;

        let limits = &context.security_context.policies.transaction_limits;
        let message = &transaction.message;

        // Check transaction limits
        if message.instructions.len() > limits.max_instructions {
            errors.push(ValidationError {
                code: ErrorCode::TooManyInstructions,
                message: format!("Too many instructions: {}", message.instructions.len()),
                component: Component::Transaction,
            });
            risk_contribution += 20;
        }

        if message.account_keys.len() > limits.max_accounts {
            errors.push(ValidationError {
                code: ErrorCode::TooManyAccounts,
                message: format!("Too many accounts: {}", message.account_keys.len()),
                component: Component::Transaction,
            });
            risk_contribution += 15;
        }

        // Check blocked accounts
        let blocked_accounts = &context.security_context.policies.account_restrictions.blocked_accounts;
        for account in &message.account_keys {
            if blocked_accounts.contains(account) {
                errors.push(ValidationError {
                    code: ErrorCode::BlockedAccount,
                    message: format!("Blocked account: {}", account),
                    component: Component::Account(*account),
                });
                risk_contribution += 50;
            }
        }

        Ok(ValidationResult {
            valid: errors.is_empty(),
            risk_level: RiskLevel::from_risk_level(risk_contribution),
            errors,
            warnings,
            security_analysis: SecurityAnalysis::new(SecurityContext::new()),
        })
    }
}

/// Security validation rule
struct SecurityValidationRule;

#[async_trait]
impl ValidationRule for SecurityValidationRule {
    fn name(&self) -> &str { "SecurityValidation" }
    fn is_critical(&self) -> bool { true }

    async fn validate(&self, transaction: &Transaction, context: &ValidationContext) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut risk_contribution = 0u8;

        let message = &transaction.message;
        let policies = &context.signing_policies;

        // Check program authorization
        for (idx, instruction) in message.instructions.iter().enumerate() {
            let program_id = message.account_keys[instruction.program_id_index as usize];

            if policies.blocked_programs.contains(&program_id) {
                errors.push(ValidationError {
                    code: ErrorCode::UnauthorizedProgram,
                    message: format!("Blocked program: {}", program_id),
                    component: Component::Instruction(idx),
                });
                risk_contribution += 40;
            }

            if let Some(allowed) = &policies.allowed_programs {
                if !allowed.contains(&program_id) {
                    errors.push(ValidationError {
                        code: ErrorCode::UnauthorizedProgram,
                        message: format!("Program not in whitelist: {}", program_id),
                        component: Component::Instruction(idx),
                    });
                    risk_contribution += 30;
                }
            }
        }

        // Check for unusual patterns
        let mut program_calls: HashMap<Pubkey, usize> = HashMap::new();
        for instruction in &message.instructions {
            let program_id = message.account_keys[instruction.program_id_index as usize];
            *program_calls.entry(program_id).or_insert(0) += 1;
        }

        for (program_id, count) in program_calls {
            if count > 3 {
                warnings.push(ValidationWarning {
                    code: WarningCode::UnusualPattern,
                    message: format!("Program {} called {} times", program_id, count),
                    severity: WarningSeverity::Medium,
                });
                risk_contribution += 10;
            }
        }

        Ok(ValidationResult {
            valid: errors.is_empty(),
            risk_level: RiskLevel::from_risk_level(risk_contribution),
            errors,
            warnings,
            security_analysis: SecurityAnalysis::new(SecurityContext::new()),
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
    async fn test_validation_result() {
        let result = ValidationResult {
            valid: true,
            risk_level: RiskLevel::Low,
            errors: Vec::new(),
            warnings: vec![ValidationWarning {
                code: WarningCode::HighValue,
                message: "High value transaction".to_string(),
                severity: WarningSeverity::Medium,
            }],
            security_analysis: SecurityAnalysis::new(SecurityContext::new()),
        };

        assert!(result.valid);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.risk_level, RiskLevel::Low);
    }
}