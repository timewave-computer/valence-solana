//! Abstract signing service interface for external signers

use crate::{Result, RuntimeError, UnsignedTransaction};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::collections::HashMap;
use std::fmt;

/// Signing backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SigningBackend {
    LocalKeypair,
    HSM,
    HardwareWallet,
    MPC,
    RemoteSigner,
    ThresholdSignature,
}

impl fmt::Display for SigningBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SigningBackend::LocalKeypair => write!(f, "LocalKeypair"),
            SigningBackend::HSM => write!(f, "HSM"),
            SigningBackend::HardwareWallet => write!(f, "HardwareWallet"),
            SigningBackend::MPC => write!(f, "MPC"),
            SigningBackend::RemoteSigner => write!(f, "RemoteSigner"),
            SigningBackend::ThresholdSignature => write!(f, "ThresholdSignature"),
        }
    }
}

/// Signing request with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningRequest {
    pub request_id: String,
    pub transaction: UnsignedTransaction,
    pub required_signers: Vec<Pubkey>,
    pub context: SigningContext,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Context for signing decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningContext {
    pub operation: String,
    pub protocol: Option<String>,
    pub risk_level: RiskLevel,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Risk level assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RiskLevel {
    #[default]
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn from_risk_level(level: u8) -> Self {
        match level {
            0..=25 => RiskLevel::Low,
            26..=50 => RiskLevel::Medium,
            51..=75 => RiskLevel::High,
            _ => RiskLevel::Critical,
        }
    }
}

/// Signing response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningResponse {
    pub request_id: String,
    pub result: SigningResult,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Result of signing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SigningResult {
    Signed {
        signatures: Vec<Signature>,
        signed_transaction: Vec<u8>,
    },
    Rejected {
        reason: String,
        policy_violations: Vec<String>,
    },
    PendingApproval {
        approval_id: String,
        required_approvers: Vec<String>,
    },
    Error { message: String },
}

/// Signature verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub valid: bool,
    pub signature_results: Vec<SignatureVerification>,
    pub policy_violations: Vec<String>,
}

/// Individual signature verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureVerification {
    pub pubkey: Pubkey,
    pub signature: Signature,
    pub valid: bool,
    pub error: Option<String>,
}

/// Abstract signing service trait
#[async_trait]
pub trait SigningService: Send + Sync {
    fn backend_type(&self) -> SigningBackend;
    async fn has_signer(&self, pubkey: &Pubkey) -> Result<bool>;
    async fn available_signers(&self) -> Result<Vec<Pubkey>>;
    async fn sign_transaction(&self, request: SigningRequest) -> Result<SigningResponse>;
    async fn verify_signatures(&self, transaction: &[u8], signatures: &[Signature], pubkeys: &[Pubkey]) -> Result<VerificationResult>;
    async fn get_signing_policies(&self, pubkey: &Pubkey) -> Result<SigningPolicies>;
    async fn update_signing_policies(&self, pubkey: &Pubkey, policies: SigningPolicies) -> Result<()>;
}

/// Signing policies for a specific key
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SigningPolicies {
    pub max_transaction_value: Option<u64>,
    pub allowed_programs: Option<Vec<Pubkey>>,
    pub blocked_programs: Vec<Pubkey>,
    pub approval_requirements: HashMap<RiskLevel, ApprovalRequirement>,
    pub rate_limit: Option<RateLimit>,
    pub time_restrictions: Option<TimeRestrictions>,
}

/// Approval requirement configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequirement {
    pub required_approvers: u32,
    pub timeout_seconds: u64,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub max_transactions: u32,
    pub window_seconds: u64,
}

/// Time-based restrictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRestrictions {
    pub allowed_days: Vec<u8>,
    pub allowed_hours: Vec<u8>,
    pub timezone: String,
}

/// Composite signing service that supports multiple backends
pub struct CompositeSigningService {
    default_backend: SigningBackend,
}

impl CompositeSigningService {
    pub fn new(default_backend: SigningBackend) -> Self {
        Self { default_backend }
    }
}

#[async_trait]
impl SigningService for CompositeSigningService {
    fn backend_type(&self) -> SigningBackend {
        self.default_backend
    }

    async fn has_signer(&self, _pubkey: &Pubkey) -> Result<bool> {
        Ok(false)
    }

    async fn available_signers(&self) -> Result<Vec<Pubkey>> {
        Ok(Vec::new())
    }

    async fn sign_transaction(&self, _request: SigningRequest) -> Result<SigningResponse> {
        Err(RuntimeError::TransactionBuildError("Signing not implemented".to_string()))
    }

    async fn verify_signatures(&self, _transaction: &[u8], _signatures: &[Signature], _pubkeys: &[Pubkey]) -> Result<VerificationResult> {
        Ok(VerificationResult {
            valid: false,
            signature_results: Vec::new(),
            policy_violations: Vec::new(),
        })
    }

    async fn get_signing_policies(&self, _pubkey: &Pubkey) -> Result<SigningPolicies> {
        Ok(SigningPolicies::default())
    }

    async fn update_signing_policies(&self, _pubkey: &Pubkey, _policies: SigningPolicies) -> Result<()> {
        Ok(())
    }
}

/// Helper functions for creating signing requests
impl SigningRequest {
    pub fn new(transaction: UnsignedTransaction, operation: String, risk_level: RiskLevel) -> Self {
        let request_id = uuid::Uuid::new_v4().to_string();
        let required_signers = transaction.signers.clone();

        Self {
            request_id,
            transaction,
            required_signers,
            context: SigningContext {
                operation,
                protocol: None,
                risk_level,
                metadata: HashMap::new(),
            },
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_protocol(mut self, protocol: String) -> Self {
        self.context.protocol = Some(protocol);
        self
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.context.metadata.insert(key, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signing_backend_display() {
        assert_eq!(SigningBackend::HSM.to_string(), "HSM");
        assert_eq!(SigningBackend::MPC.to_string(), "MPC");
    }

    #[test]
    fn test_signing_request_creation() {
        let unsigned_tx = UnsignedTransaction {
            message: vec![1, 2, 3],
            recent_blockhash: solana_sdk::hash::Hash::default(),
            signers: vec![Pubkey::new_unique()],
            metadata: crate::TransactionMetadata {
                description: "Test".to_string(),
                compute_units: None,
                priority_fee: None,
                simulation: None,
            },
        };

        let request = SigningRequest::new(unsigned_tx, "test_operation".to_string(), RiskLevel::Low);

        assert_eq!(request.context.operation, "test_operation");
        assert_eq!(request.context.risk_level, RiskLevel::Low);
        assert!(request.context.protocol.is_none());
    }

    #[tokio::test]
    async fn test_composite_service() {
        let service = CompositeSigningService::new(SigningBackend::LocalKeypair);
        let signers = service.available_signers().await.unwrap();
        assert!(signers.is_empty());
    }
}