/// Testing infrastructure for Valence Protocol SDK
/// This module provides testing utilities and helpers for SDK development and testing

use anchor_lang::prelude::*;
use crate::{ValenceExecutionContext, SessionMetadata, Session};

/// PDA utilities for testing
pub mod pda_utils {
    use super::*;
    
    /// Find a PDA for testing
    pub fn find_test_pda(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(seeds, program_id)
    }
    
    /// Create test seeds
    pub fn create_test_seeds(base: &str, suffix: &str) -> Vec<Vec<u8>> {
        vec![
            base.as_bytes().to_vec(),
            suffix.as_bytes().to_vec(),
        ]
    }
}

/// Mock data generation for testing
pub mod mock_data {
    use super::*;
    
    /// Generate a mock capability ID
    pub fn mock_capability_id() -> String {
        "test_capability_12345".to_string()
    }
    
    /// Generate mock session metadata
    pub fn mock_session_metadata() -> SessionMetadata {
        SessionMetadata {
            description: "Test session".to_string(),
            tags: vec!["test".to_string(), "mock".to_string()],
            max_lifetime: 3600, // 1 hour
        }
    }
    
    /// Generate mock session
    pub fn mock_session() -> Session {
        Session {
            session_id: "test_session_123".to_string(),
            owner: mock_pubkey(),
            is_active: true,
            capabilities: vec!["test_capability".to_string()],
            namespaces: vec!["test_namespace".to_string()],
            metadata: mock_session_metadata(),
            created_at: 1234567890,
            last_updated: 1234567890,
            version: 1,
        }
    }
    
    /// Generate mock pubkey
    pub fn mock_pubkey() -> Pubkey {
        Pubkey::new_unique()
    }
    
    /// Generate mock execution context
    pub fn mock_execution_context() -> ValenceExecutionContext {
        ValenceExecutionContext::new(
            mock_capability_id(),
            mock_pubkey(),
            mock_pubkey(),
        )
        .with_input_data(vec![1, 2, 3, 4, 5])
        .with_compute_limit(200_000)
        .with_labels(vec!["test".to_string(), "mock".to_string()])
    }
}

/// Context builders for testing
pub mod context_builders {
    use super::*;
    
    /// Build a test context for capability execution
    pub fn build_capability_context() -> TestContext {
        TestContext {
            capability_id: mock_data::mock_capability_id(),
            session_id: "test_session".to_string(),
            caller: mock_data::mock_pubkey(),
            program_id: mock_data::mock_pubkey(),
        }
    }
    
    /// Build a test context for session operations
    pub fn build_session_context() -> TestContext {
        TestContext {
            capability_id: "session_management".to_string(),
            session_id: "test_session_456".to_string(),
            caller: mock_data::mock_pubkey(),
            program_id: mock_data::mock_pubkey(),
        }
    }
}

/// Integration test helpers
pub mod integration_helpers {
    use super::*;
    
    /// Setup test environment
    pub fn setup_test_environment() -> TestEnvironment {
        TestEnvironment {
            program_id: mock_data::mock_pubkey(),
            authority: mock_data::mock_pubkey(),
            payer: mock_data::mock_pubkey(),
            test_accounts: vec![],
        }
    }
    
    /// Create test accounts
    pub fn create_test_accounts(count: u32) -> Vec<TestAccount> {
        (0..count)
            .map(|i| TestAccount {
                pubkey: mock_data::mock_pubkey(),
                is_signer: i == 0,
                is_writable: true,
                lamports: 1_000_000,
                data: vec![],
            })
            .collect()
    }
    
    /// Verify test results
    pub fn verify_test_results(expected: &TestResult, actual: &TestResult) -> bool {
        expected.success == actual.success
            && expected.error_code == actual.error_code
            && expected.logs.len() == actual.logs.len()
    }
}

/// Test context structure
#[derive(Debug, Clone)]
pub struct TestContext {
    pub capability_id: String,
    pub session_id: String,
    pub caller: Pubkey,
    pub program_id: Pubkey,
}

/// Test environment structure
#[derive(Debug, Clone)]
pub struct TestEnvironment {
    pub program_id: Pubkey,
    pub authority: Pubkey,
    pub payer: Pubkey,
    pub test_accounts: Vec<TestAccount>,
}

/// Test account structure
#[derive(Debug, Clone)]
pub struct TestAccount {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
    pub lamports: u64,
    pub data: Vec<u8>,
}

/// Test result structure
#[derive(Debug, Clone, PartialEq)]
pub struct TestResult {
    pub success: bool,
    pub error_code: Option<u32>,
    pub logs: Vec<String>,
    pub return_data: Option<Vec<u8>>,
}

impl TestResult {
    /// Create a successful test result
    pub fn success() -> Self {
        Self {
            success: true,
            error_code: None,
            logs: vec![],
            return_data: None,
        }
    }
    
    /// Create a failed test result
    pub fn failure(error_code: u32) -> Self {
        Self {
            success: false,
            error_code: Some(error_code),
            logs: vec![],
            return_data: None,
        }
    }
    
    /// Add a log message
    pub fn with_log(mut self, message: String) -> Self {
        self.logs.push(message);
        self
    }
    
    /// Add return data
    pub fn with_return_data(mut self, data: Vec<u8>) -> Self {
        self.return_data = Some(data);
        self
    }
}
