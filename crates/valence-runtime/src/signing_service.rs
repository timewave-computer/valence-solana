//! Abstract signing service interface for external signers
//!
//! This module provides a secure abstraction layer between transaction construction
//! and signing, supporting various signing backends including HSMs, hardware wallets,
//! and MPC systems.

use crate::{Result, RuntimeError, UnsignedTransaction};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use tokio::sync::RwLock;
use tracing::info;

/// Signing backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SigningBackend {
    /// Local file-based keypair (development only)
    LocalKeypair,
    /// Hardware Security Module
    HSM,
    /// Hardware wallet (Ledger, etc.)
    HardwareWallet,
    /// Multi-party computation
    MPC,
    /// Remote signing service
    RemoteSigner,
    /// Threshold signature scheme
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
    /// Unique request ID
    pub request_id: String,

    /// Transaction to sign
    pub transaction: UnsignedTransaction,

    /// Required signers
    pub required_signers: Vec<Pubkey>,

    /// Signing context
    pub context: SigningContext,

    /// Request timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Context for signing decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningContext {
    /// Operation type
    pub operation: String,

    /// Protocol or program involved
    pub protocol: Option<String>,

    /// Risk assessment
    pub risk_level: RiskLevel,

    /// Additional metadata
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
    /// Request ID
    pub request_id: String,

    /// Signing result
    pub result: SigningResult,

    /// Response timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Result of signing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SigningResult {
    /// Successfully signed
    Signed {
        signatures: Vec<Signature>,
        signed_transaction: Vec<u8>,
    },
    /// Rejected by policy
    Rejected {
        reason: String,
        policy_violations: Vec<String>,
    },
    /// Requires additional approval
    PendingApproval {
        approval_id: String,
        required_approvers: Vec<String>,
    },
    /// Error during signing
    Error { message: String },
}

/// Signature verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether all signatures are valid
    pub valid: bool,

    /// Individual signature verification results
    pub signature_results: Vec<SignatureVerification>,

    /// Any policy violations detected
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
    /// Get the signing backend type
    fn backend_type(&self) -> SigningBackend;

    /// Check if a pubkey is available for signing
    async fn has_signer(&self, pubkey: &Pubkey) -> Result<bool>;

    /// Get all available signing pubkeys
    async fn available_signers(&self) -> Result<Vec<Pubkey>>;

    /// Sign a transaction
    async fn sign_transaction(&self, request: SigningRequest) -> Result<SigningResponse>;

    /// Verify signatures on a transaction
    async fn verify_signatures(
        &self,
        transaction: &[u8],
        signatures: &[Signature],
        pubkeys: &[Pubkey],
    ) -> Result<VerificationResult>;

    /// Get signing policies for a pubkey
    async fn get_signing_policies(&self, pubkey: &Pubkey) -> Result<SigningPolicies>;

    /// Update signing policies
    async fn update_signing_policies(
        &self,
        pubkey: &Pubkey,
        policies: SigningPolicies,
    ) -> Result<()>;
}

/// Signing policies for a specific key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningPolicies {
    /// Maximum transaction value (lamports)
    pub max_transaction_value: Option<u64>,

    /// Allowed programs
    pub allowed_programs: Option<Vec<Pubkey>>,

    /// Blocked programs
    pub blocked_programs: Vec<Pubkey>,

    /// Required approvals for high-risk operations
    pub approval_requirements: HashMap<RiskLevel, ApprovalRequirement>,

    /// Rate limiting
    pub rate_limit: Option<RateLimit>,

    /// Time-based restrictions
    pub time_restrictions: Option<TimeRestrictions>,
}

impl Default for SigningPolicies {
    fn default() -> Self {
        let mut approval_requirements = HashMap::new();
        approval_requirements.insert(
            RiskLevel::Critical,
            ApprovalRequirement {
                required_approvers: 2,
                timeout_seconds: 3600,
            },
        );

        Self {
            max_transaction_value: None,
            allowed_programs: None,
            blocked_programs: Vec::new(),
            approval_requirements,
            rate_limit: None,
            time_restrictions: None,
        }
    }
}

/// Approval requirement configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequirement {
    /// Number of required approvals
    pub required_approvers: u32,

    /// Timeout for approvals (seconds)
    pub timeout_seconds: u64,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Maximum transactions per time window
    pub max_transactions: u32,

    /// Time window in seconds
    pub window_seconds: u64,
}

/// Time-based restrictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRestrictions {
    /// Allowed days of week (0 = Sunday, 6 = Saturday)
    pub allowed_days: Vec<u8>,

    /// Allowed hours (0-23)
    pub allowed_hours: Vec<u8>,

    /// Timezone for restrictions
    pub timezone: String,
}

/// Composite signing service that supports multiple backends
pub struct CompositeSigningService {
    /// Available signing backends
    backends: RwLock<HashMap<SigningBackend, SigningBackend>>, // Simplified to just track backend types

    /// Default backend
    default_backend: SigningBackend,

    /// Backend selection strategy
    backend_selector: DefaultBackendSelector, // Use concrete type instead of trait object
}

/// Backend selection strategy
#[async_trait]
pub trait BackendSelector: Send + Sync {
    /// Select appropriate backend for a signing request
    async fn select_backend(
        &self,
        request: &SigningRequest,
        available_backends: &[SigningBackend],
    ) -> Result<SigningBackend>;
}

/// Default backend selector
pub struct DefaultBackendSelector;

#[async_trait]
impl BackendSelector for DefaultBackendSelector {
    async fn select_backend(
        &self,
        request: &SigningRequest,
        available_backends: &[SigningBackend],
    ) -> Result<SigningBackend> {
        // Select based on risk level
        match request.context.risk_level {
            RiskLevel::Critical => {
                // Prefer HSM or MPC for critical operations
                if available_backends.contains(&SigningBackend::HSM) {
                    Ok(SigningBackend::HSM)
                } else if available_backends.contains(&SigningBackend::MPC) {
                    Ok(SigningBackend::MPC)
                } else {
                    Err(RuntimeError::TransactionBuildError(
                        "No secure backend available for critical operation".to_string(),
                    ))
                }
            }
            RiskLevel::High => {
                // Prefer hardware wallet or HSM
                if available_backends.contains(&SigningBackend::HardwareWallet) {
                    Ok(SigningBackend::HardwareWallet)
                } else if available_backends.contains(&SigningBackend::HSM) {
                    Ok(SigningBackend::HSM)
                } else {
                    Ok(available_backends[0])
                }
            }
            _ => {
                // Use any available backend
                Ok(available_backends[0])
            }
        }
    }
}

impl CompositeSigningService {
    /// Create a new composite signing service
    pub fn new(default_backend: SigningBackend) -> Self {
        Self {
            backends: RwLock::new(HashMap::new()),
            default_backend,
            backend_selector: DefaultBackendSelector,
        }
    }

    /// Create with custom backend selector
    pub fn with_selector(
        default_backend: SigningBackend,
        selector: DefaultBackendSelector,
    ) -> Self {
        Self {
            backends: RwLock::new(HashMap::new()),
            default_backend,
            backend_selector: selector,
        }
    }

    /// Get the backend selector
    pub fn backend_selector(&self) -> &DefaultBackendSelector {
        &self.backend_selector
    }

    /// Register a signing backend
    pub async fn register_backend(
        &self,
        backend_type: SigningBackend,
        backend: SigningBackend,
    ) -> Result<()> {
        let mut backends = self.backends.write().await;
        backends.insert(backend_type, backend);
        info!("Registered signing backend: {}", backend_type);
        Ok(())
    }

    /// Remove a signing backend
    pub async fn remove_backend(&self, backend_type: SigningBackend) -> Result<()> {
        let mut backends = self.backends.write().await;
        backends.remove(&backend_type);
        info!("Removed signing backend: {}", backend_type);
        Ok(())
    }

    /// Get available backends
    pub async fn available_backends(&self) -> Vec<SigningBackend> {
        let backends = self.backends.read().await;
        backends.keys().cloned().collect()
    }
}

#[async_trait]
impl SigningService for CompositeSigningService {
    fn backend_type(&self) -> SigningBackend {
        self.default_backend
    }

    async fn has_signer(&self, _pubkey: &Pubkey) -> Result<bool> {
        // Simplified implementation - in a real system this would check actual signing backends
        Ok(false)
    }

    async fn available_signers(&self) -> Result<Vec<Pubkey>> {
        // Simplified implementation - return empty list
        Ok(Vec::new())
    }

    async fn sign_transaction(&self, _request: SigningRequest) -> Result<SigningResponse> {
        // Simplified implementation - return error for now
        Err(RuntimeError::TransactionBuildError(
            "Signing not implemented".to_string(),
        ))
    }

    async fn verify_signatures(
        &self,
        _transaction: &[u8],
        _signatures: &[Signature],
        _pubkeys: &[Pubkey],
    ) -> Result<VerificationResult> {
        // Simplified implementation - return empty verification
        Ok(VerificationResult {
            valid: false,
            signature_results: Vec::new(),
            policy_violations: Vec::new(),
        })
    }

    async fn get_signing_policies(&self, _pubkey: &Pubkey) -> Result<SigningPolicies> {
        // Simplified implementation - return default policies
        Ok(SigningPolicies {
            max_transaction_value: None,
            allowed_programs: None,
            blocked_programs: Vec::new(),
            approval_requirements: std::collections::HashMap::new(),
            rate_limit: None,
            time_restrictions: None,
        })
    }

    async fn update_signing_policies(
        &self,
        _pubkey: &Pubkey,
        _policies: SigningPolicies,
    ) -> Result<()> {
        // Simplified implementation - policies update not implemented
        Ok(())
    }
}

/// Helper functions for creating signing requests
impl SigningRequest {
    /// Create a new signing request
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

    /// Add protocol context
    pub fn with_protocol(mut self, protocol: String) -> Self {
        self.context.protocol = Some(protocol);
        self
    }

    /// Add metadata
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

        let request =
            SigningRequest::new(unsigned_tx, "test_operation".to_string(), RiskLevel::Low);

        assert_eq!(request.context.operation, "test_operation");
        assert_eq!(request.context.risk_level, RiskLevel::Low);
        assert!(request.context.protocol.is_none());
    }

    #[tokio::test]
    async fn test_composite_service() {
        let service = CompositeSigningService::new(SigningBackend::LocalKeypair);
        let backends = service.available_backends().await;
        assert!(backends.is_empty());
    }
}
