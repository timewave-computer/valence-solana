//! Security utilities and validation

use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, transaction::Transaction};
use std::collections::{HashMap, HashSet};

pub mod audit;
pub mod validation;
pub mod signing;

pub use audit::{AuditEntry, AuditLogger};
pub use validation::{TransactionValidator, ValidationResult, ValidationRule};
pub use signing::{SigningService, CompositeSigningService, SigningRequest, SigningResponse};

/// Security context for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub environment: Environment,
    pub policies: SecurityPolicies,
    pub session: Option<SessionInfo>,
}

impl SecurityContext {
    pub fn new() -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            environment: Environment::Development,
            policies: SecurityPolicies::default(),
            session: None,
        }
    }
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Operating environment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Staging => write!(f, "staging"),
            Environment::Production => write!(f, "production"),
        }
    }
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub identity: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Security policies configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityPolicies {
    pub transaction_limits: TransactionLimits,
    pub program_restrictions: ProgramRestrictions,
    pub account_restrictions: AccountRestrictions,
    pub rate_limits: RateLimits,
}

/// Transaction limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLimits {
    pub max_size_bytes: usize,
    pub max_instructions: usize,
    pub max_compute_units: u32,
    pub max_value_lamports: u64,
    pub max_accounts: usize,
}

impl Default for TransactionLimits {
    fn default() -> Self {
        Self {
            max_size_bytes: 1232, // Solana max transaction size
            max_instructions: 20,
            max_compute_units: 1_400_000,
            max_value_lamports: 1_000_000_000_000, // 1000 SOL
            max_accounts: 32,
        }
    }
}

/// Program restrictions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProgramRestrictions {
    pub allowed_programs: Option<HashSet<Pubkey>>,
    pub blocked_programs: HashSet<Pubkey>,
    pub require_verified_programs: bool,
}

/// Account restrictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountRestrictions {
    pub blocked_accounts: HashSet<Pubkey>,
    pub require_known_owners: bool,
    pub max_writable_accounts: usize,
}

impl Default for AccountRestrictions {
    fn default() -> Self {
        Self {
            blocked_accounts: HashSet::new(),
            require_known_owners: false,
            max_writable_accounts: 16,
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    pub transactions_per_minute: u32,
    pub transactions_per_hour: u32,
    pub compute_units_per_hour: u64,
    pub value_per_day: u64,
}

impl Default for RateLimits {
    fn default() -> Self {
        Self {
            transactions_per_minute: 60,
            transactions_per_hour: 1000,
            compute_units_per_hour: 1_000_000_000,
            value_per_day: 10_000_000_000_000, // 10,000 SOL
        }
    }
}

/// Security analyzer for transactions
pub struct SecurityAnalyzer {
    known_programs: HashMap<Pubkey, ProgramInfo>,
}

/// Program information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramInfo {
    pub name: String,
    pub verified: bool,
    pub risk_level: crate::security::signing::RiskLevel,
    pub vulnerabilities: Vec<String>,
}

impl SecurityAnalyzer {
    pub fn new(_context: SecurityContext) -> Self {
        let mut known_programs = HashMap::new();

        // Add well-known programs
        known_programs.insert(
            solana_sdk::system_program::id(),
            ProgramInfo {
                name: "System Program".to_string(),
                verified: true,
                risk_level: crate::security::signing::RiskLevel::Low,
                vulnerabilities: Vec::new(),
            },
        );

        known_programs.insert(
            spl_token::id(),
            ProgramInfo {
                name: "Token Program".to_string(),
                verified: true,
                risk_level: crate::security::signing::RiskLevel::Low,
                vulnerabilities: Vec::new(),
            },
        );

        Self { known_programs }
    }

    pub fn add_known_program(&mut self, program_id: Pubkey, info: ProgramInfo) {
        self.known_programs.insert(program_id, info);
    }

    pub fn analyze_transaction(&self, transaction: &Transaction) -> SecurityAnalysis {
        let mut analysis = SecurityAnalysis::default();

        // Analyze programs
        for instruction in &transaction.message.instructions {
            let program_id = transaction.message.account_keys[instruction.program_id_index as usize];

            if let Some(info) = self.known_programs.get(&program_id) {
                analysis.programs.push(AnalyzedProgram {
                    program_id,
                    name: info.name.clone(),
                    verified: info.verified,
                    risk_level: info.risk_level,
                    vulnerabilities: info.vulnerabilities.clone(),
                });

                if info.risk_level as u8 > analysis.overall_risk_level as u8 {
                    analysis.overall_risk_level = info.risk_level;
                }
            } else {
                analysis.programs.push(AnalyzedProgram {
                    program_id,
                    name: "Unknown Program".to_string(),
                    verified: false,
                    risk_level: crate::security::signing::RiskLevel::High,
                    vulnerabilities: vec!["Unknown program".to_string()],
                });
                analysis.overall_risk_level = crate::security::signing::RiskLevel::High;
            }
        }

        analysis
    }

    pub fn calculate_risk_score(&self, analysis: &SecurityAnalysis) -> u8 {
        let mut score = 0u8;

        // Base score from risk level
        score += match analysis.overall_risk_level {
            crate::security::signing::RiskLevel::Low => 10,
            crate::security::signing::RiskLevel::Medium => 30,
            crate::security::signing::RiskLevel::High => 60,
            crate::security::signing::RiskLevel::Critical => 80,
        };

        // Add points for unknown programs
        let unknown_programs = analysis.programs.iter().filter(|p| !p.verified).count() as u8;
        score = score.saturating_add(unknown_programs * 10);

        score.min(100)
    }
}

/// Security analysis result
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityAnalysis {
    pub overall_risk_level: crate::security::signing::RiskLevel,
    pub programs: Vec<AnalyzedProgram>,
    pub writable_accounts: Vec<Pubkey>,
}

impl SecurityAnalysis {
    pub fn new(_context: SecurityContext) -> Self {
        Self {
            overall_risk_level: crate::security::signing::RiskLevel::Low,
            programs: Vec::new(),
            writable_accounts: Vec::new(),
        }
    }
}

/// Analyzed program information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzedProgram {
    pub program_id: Pubkey,
    pub name: String,
    pub verified: bool,
    pub risk_level: crate::security::signing::RiskLevel,
    pub vulnerabilities: Vec<String>,
}

/// Security utilities
pub mod utils {
    use super::*;
    use sha2::{Digest, Sha256};

    pub fn transaction_hash(tx: &Transaction) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bincode::serialize(&tx.message).unwrap_or_default());
        format!("{:x}", hasher.finalize())
    }

    pub fn is_system_account(pubkey: &Pubkey) -> bool {
        pubkey == &solana_sdk::system_program::id()
            || pubkey == &spl_token::id()
            || pubkey == &spl_associated_token_account::id()
    }

    pub fn sanitize_for_log(input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
            .take(256)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_display() {
        assert_eq!(Environment::Production.to_string(), "production");
        assert_eq!(Environment::Development.to_string(), "development");
    }

    #[test]
    fn test_default_policies() {
        let policies = SecurityPolicies::default();
        assert_eq!(policies.transaction_limits.max_instructions, 20);
        assert_eq!(policies.rate_limits.transactions_per_minute, 60);
    }

    #[test]
    fn test_security_analyzer() {
        let context = SecurityContext {
            timestamp: chrono::Utc::now(),
            environment: Environment::Development,
            policies: SecurityPolicies::default(),
            session: None,
        };

        let analyzer = SecurityAnalyzer::new(context);
        assert!(analyzer.known_programs.contains_key(&solana_sdk::system_program::id()));
    }
}