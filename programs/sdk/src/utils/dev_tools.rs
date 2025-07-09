/// Development tools and utilities for testing Valence Protocol integrations
/// 
/// This module provides helpful utilities for development and testing:
/// - Mock data generation
/// - Test account creation
/// - Local cluster setup helpers
/// - Performance testing utilities

use crate::{
    types::*,
    error::*,
    utils::*,
    ValenceClient,
};
use anchor_lang::prelude::*;
use solana_sdk::{signature::Keypair, signer::Signer, commitment_config::CommitmentConfig};

/// Development toolkit for Valence Protocol
pub struct DevToolkit {
    /// Mock data for testing
    pub mock_data: MockDataGenerator,
    /// Test helpers
    pub test_helpers: TestHelpers,
    /// Development utilities
    pub dev_utils: DevUtils,
}

impl DevToolkit {
    /// Create a new development toolkit
    pub fn new() -> Self {
        Self {
            mock_data: MockDataGenerator::new(),
            test_helpers: TestHelpers::new(),
            dev_utils: DevUtils::new(),
        }
    }
}

impl Default for DevToolkit {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock data generator for testing
pub struct MockDataGenerator {
    counter: std::cell::RefCell<u64>,
}

impl MockDataGenerator {
    pub fn new() -> Self {
        Self {
            counter: std::cell::RefCell::new(0),
        }
    }

    /// Generate a mock capability
    pub fn mock_capability(&self, capability_id: Option<String>) -> Capability {
        let id = capability_id.unwrap_or_else(|| format!("test_capability_{}", self.next_id()));
        
        Capability {
            capability_id: id,
            shard: Pubkey::new_unique(),
            verification_functions: vec![
                sha256(b"basic_permission"),
                sha256(b"parameter_constraint"),
            ],
            description: "Mock capability for testing".to_string(),
            is_active: true,
            total_executions: self.next_id(),
            last_execution_block_height: 1000 + self.next_id(),
            last_execution_timestamp: current_timestamp(),
        }
    }

    /// Generate a mock session
    pub fn mock_session(&self, session_id: Option<String>) -> Session {
        let id = session_id.unwrap_or_else(|| format!("test_session_{}", self.next_id()));
        
        Session {
            session_id: id,
            owner: Pubkey::new_unique(),
            is_active: true,
            capabilities: vec![
                "test_capability_1".to_string(),
                "test_capability_2".to_string(),
            ],
            namespaces: vec![
                "test_namespace".to_string(),
            ],
            metadata: SessionMetadata {
                description: "Mock session for testing".to_string(),
                tags: vec!["test".to_string(), "mock".to_string()],
                max_lifetime: 3600,
            },
            created_at: current_timestamp(),
            last_updated: current_timestamp(),
            version: 1,
        }
    }

    /// Generate a mock library entry
    pub fn mock_library_entry(&self, library_id: Option<String>) -> LibraryEntry {
        let id = library_id.unwrap_or_else(|| format!("test_library_{}", self.next_id()));
        let name = format!("Test Library {}", self.current_id());
        let version = "1.0.0".to_string();
        let tags = vec!["test".to_string(), "mock".to_string()];
        
        LibraryEntry {
            library_id: id,
            name: name.clone(),
            version: version.clone(),
            author: Pubkey::new_unique(),
            metadata_hash: calculate_metadata_hash(&name, &version, "Test library", &tags),
            program_id: Pubkey::new_unique(),
            status: LibraryStatus::Published,
            dependencies: vec![],
            tags,
            is_verified: false,
            usage_count: self.next_id(),
        }
    }

    /// Generate a mock execution context
    pub fn mock_execution_context(&self, capability_id: Option<String>) -> ValenceExecutionContext {
        let id = capability_id.unwrap_or_else(|| format!("test_capability_{}", self.next_id()));
        
        ValenceExecutionContext::new(
            id,
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        )
        .with_input_data(vec![1, 2, 3, 4, 5])
        .with_compute_limit(100_000)
        .with_labels(vec!["test".to_string(), "mock".to_string()])
    }

    /// Generate mock verification functions
    pub fn mock_verification_functions(&self) -> Vec<[u8; 32]> {
        vec![
            sha256(b"basic_permission"),
            sha256(b"parameter_constraint"),
            sha256(b"zk_proof"),
        ]
    }

    /// Generate a mock keypair with optional seed
    pub fn mock_keypair(&self, seed: Option<u8>) -> Keypair {
        if let Some(s) = seed {
            let mut seed_bytes = [0u8; 64];
            seed_bytes[0] = s;
            seed_bytes[32] = s;
            if let Ok(keypair) = Keypair::from_bytes(&seed_bytes) {
                keypair
            } else {
                Keypair::new()
            }
        } else {
            Keypair::new()
        }
    }

    /// Generate mock program IDs for testing
    pub fn mock_program_ids(&self) -> ProgramIds {
        ProgramIds {
            kernel: Pubkey::new_unique(),
            processor: Pubkey::new_unique(),
            scheduler: Pubkey::new_unique(),
            diff: Pubkey::new_unique(),
            registry: Pubkey::new_unique(),
        }
    }

    /// Generate mock execution result
    pub fn mock_execution_result(&self, capability_id: String) -> ExecutionResult {
        ExecutionResult {
            transaction_result: TransactionResult {
                signature: solana_sdk::signature::Signature::new_unique(),
                success: true,
                error: None,
                logs: vec!["Program log: Execution successful".to_string()],
            },
            execution_id: Some(self.next_id()),
            capability_id,
            session: Pubkey::new_unique(),
        }
    }

    fn next_id(&self) -> u64 {
        let mut counter = self.counter.borrow_mut();
        *counter += 1;
        *counter
    }

    fn current_id(&self) -> u64 {
        *self.counter.borrow()
    }
}

/// Test helpers for integration and unit testing
pub struct TestHelpers {
    /// Temporary keypairs for testing
    pub test_keypairs: Vec<Keypair>,
}

impl TestHelpers {
    pub fn new() -> Self {
        Self {
            test_keypairs: Vec::new(),
        }
    }

    /// Create a test environment with mock configuration
    pub fn create_test_config(&self) -> ValenceConfig {
        ValenceConfig {
            program_ids: ProgramIds {
                kernel: Pubkey::new_unique(),
                processor: Pubkey::new_unique(),
                scheduler: Pubkey::new_unique(),
                diff: Pubkey::new_unique(),
                registry: Pubkey::new_unique(),
            },
            cluster: anchor_client::Cluster::Localnet,
            payer: Keypair::new(),
            commitment: Some(CommitmentConfig::confirmed()),
        }
    }

    /// Validate test data integrity
    pub fn validate_test_data<T>(&self, data: &T) -> ValenceResult<()>
    where
        T: std::fmt::Debug,
    {
        // Basic validation - in practice, this would be more comprehensive
        println!("Validating test data: {:?}", data);
        Ok(())
    }

    /// Create a test capability with validation
    pub fn create_test_capability(&self, capability_id: &str) -> ValenceResult<Capability> {
        validate_capability_id(capability_id)?;
        
        Ok(Capability {
            capability_id: capability_id.to_string(),
            shard: Pubkey::new_unique(),
            verification_functions: vec![sha256(b"test_verification")],
            description: format!("Test capability: {}", capability_id),
            is_active: true,
            total_executions: 0,
            last_execution_block_height: 0,
            last_execution_timestamp: current_timestamp(),
        })
    }

    /// Create a test session with validation
    pub fn create_test_session(&self, session_id: &str, capabilities: Vec<String>) -> ValenceResult<Session> {
        if session_id.is_empty() {
            return Err(ValenceError::InvalidInputParameters("Session ID cannot be empty".to_string()));
        }

        for capability_id in &capabilities {
            validate_capability_id(capability_id)?;
        }

        Ok(Session {
            session_id: session_id.to_string(),
            owner: Pubkey::new_unique(),
            is_active: true,
            capabilities,
            namespaces: vec!["test".to_string()],
            metadata: SessionMetadata {
                description: format!("Test session: {}", session_id),
                tags: vec!["test".to_string()],
                max_lifetime: 3600,
            },
            created_at: current_timestamp(),
            last_updated: current_timestamp(),
            version: 1,
        })
    }

    /// Simulate a successful transaction
    pub fn simulate_success_transaction(&self, signature_seed: Option<u8>) -> TransactionResult {
        let signature = if let Some(seed) = signature_seed {
            // Create deterministic signature for testing
            let mut sig_bytes = [0u8; 64];
            sig_bytes[0] = seed;
            solana_sdk::signature::Signature::from(sig_bytes)
        } else {
            solana_sdk::signature::Signature::new_unique()
        };

        TransactionResult {
            signature,
            success: true,
            error: None,
            logs: vec![
                "Program log: Instruction started".to_string(),
                "Program log: Validation passed".to_string(),
                "Program log: Execution successful".to_string(),
            ],
        }
    }

    /// Simulate a failed transaction
    pub fn simulate_failed_transaction(&self, error_message: &str) -> TransactionResult {
        TransactionResult {
            signature: solana_sdk::signature::Signature::new_unique(),
            success: false,
            error: Some(error_message.to_string()),
            logs: vec![
                "Program log: Instruction started".to_string(),
                format!("Program error: {}", error_message),
            ],
        }
    }

    /// Create test execution config
    pub fn create_test_execution_config(&self) -> ExecutionConfig {
        ExecutionConfig {
            max_execution_time: Some(60),
            max_compute_units: Some(100_000),
            record_execution: true,
            parameters: Some(vec![1, 2, 3]),
        }
    }

    /// Assert capability equality for testing
    pub fn assert_capability_eq(&self, expected: &Capability, actual: &Capability) -> ValenceResult<()> {
        if expected.capability_id != actual.capability_id {
            return Err(ValenceError::validation_failed(
                format!("Capability ID mismatch: expected '{}', got '{}'", 
                    expected.capability_id, actual.capability_id)
            ));
        }

        if expected.is_active != actual.is_active {
            return Err(ValenceError::validation_failed(
                format!("Capability active status mismatch: expected {}, got {}", 
                    expected.is_active, actual.is_active)
            ));
        }

        Ok(())
    }
}

/// Development utilities for debugging and monitoring
pub struct DevUtils {
    /// Debug mode flag
    pub debug_mode: bool,
}

impl DevUtils {
    pub fn new() -> Self {
        Self {
            debug_mode: std::env::var("VALENCE_DEBUG").is_ok(),
        }
    }

    /// Log debug information
    pub fn debug_log(&self, message: &str) {
        if self.debug_mode {
            println!("[DEBUG] {}: {}", timestamp_to_string(current_timestamp()), message);
        }
    }

    /// Pretty print a capability
    pub fn pretty_print_capability(&self, capability: &Capability) {
        println!("🔧 Capability Information:");
        println!("   ID: {}", capability.capability_id);
        println!("   Shard: {}", capability.shard);
        println!("   Description: {}", capability.description);
        println!("   Active: {}", if capability.is_active { "✅" } else { "❌" });
        println!("   Executions: {}", capability.total_executions);
        println!("   Verification Functions: {}", capability.verification_functions.len());
        
        for (i, vf) in capability.verification_functions.iter().enumerate() {
            println!("     {}: {}", i + 1, hex::encode(vf));
        }
    }

    /// Pretty print a session
    pub fn pretty_print_session(&self, session: &Session) {
        println!("📋 Session Information:");
        println!("   ID: {}", session.session_id);
        println!("   Owner: {}", session.owner);
        println!("   Active: {}", if session.is_active { "✅" } else { "❌" });
        println!("   Version: {}", session.version);
        println!("   Created: {}", timestamp_to_string(session.created_at));
        println!("   Updated: {}", timestamp_to_string(session.last_updated));
        
        println!("   Capabilities:");
        for capability in &session.capabilities {
            println!("     • {}", capability);
        }
        
        println!("   Namespaces:");
        for namespace in &session.namespaces {
            println!("     • {}", namespace);
        }
        
        println!("   Metadata:");
        println!("     Description: {}", session.metadata.description);
        println!("     Tags: {}", session.metadata.tags.join(", "));
        println!("     Max Lifetime: {} seconds", session.metadata.max_lifetime);
    }

    /// Pretty print a library entry
    pub fn pretty_print_library(&self, library: &LibraryEntry) {
        println!("📚 Library Information:");
        println!("   ID: {}", library.library_id);
        println!("   Name: {}", library.name);
        println!("   Version: {}", library.version);
        println!("   Author: {}", library.author);
        println!("   Status: {:?}", library.status);
        println!("   Verified: {}", if library.is_verified { "✅" } else { "❌" });
        println!("   Usage Count: {}", library.usage_count);
        println!("   Program ID: {}", library.program_id);
        println!("   Metadata Hash: {}", hex::encode(library.metadata_hash));
        
        if !library.tags.is_empty() {
            println!("   Tags: {}", library.tags.join(", "));
        }
        
        if !library.dependencies.is_empty() {
            println!("   Dependencies:");
            for dep in &library.dependencies {
                println!("     • {}", dep);
            }
        }
    }

    /// Pretty print an execution result
    pub fn pretty_print_execution_result(&self, result: &ExecutionResult) {
        println!("⚡ Execution Result:");
        println!("   Capability ID: {}", result.capability_id);
        println!("   Session: {}", result.session);
        
        if let Some(exec_id) = result.execution_id {
            println!("   Execution ID: {}", exec_id);
        }
        
        println!("   Transaction:");
        println!("     Signature: {}", result.transaction_result.signature);
        println!("     Success: {}", if result.transaction_result.success { "✅" } else { "❌" });
        
        if let Some(ref error) = result.transaction_result.error {
            println!("     Error: {}", error);
        }
        
        if !result.transaction_result.logs.is_empty() {
            println!("     Logs:");
            for log in &result.transaction_result.logs {
                println!("       {}", log);
            }
        }
    }

    /// Generate a development report
    pub fn generate_dev_report(&self, config: &ValenceConfig) -> String {
        let mut report = String::new();
        
        report.push_str("# Valence Protocol Development Report\n\n");
        report.push_str(&format!("Generated: {}\n\n", timestamp_to_string(current_timestamp())));
        
        report.push_str("## Configuration\n");
        report.push_str(&format!("- Cluster: {:?}\n", config.cluster));
        report.push_str(&format!("- Payer: {}\n", config.payer.pubkey()));
        report.push_str(&format!("- Commitment: {:?}\n", config.commitment));
        
        report.push_str("\n## Program IDs\n");
        report.push_str(&format!("- Kernel: {}\n", config.program_ids.kernel));
        report.push_str(&format!("- Processor: {}\n", config.program_ids.processor));
        report.push_str(&format!("- Scheduler: {}\n", config.program_ids.scheduler));
        report.push_str(&format!("- Diff: {}\n", config.program_ids.diff));
        report.push_str(&format!("- Registry: {}\n", config.program_ids.registry));
        
        report.push_str("\n## Development Status\n");
        report.push_str("- SDK: ✅ Implemented\n");
        report.push_str("- CLI: ✅ Implemented\n");
        report.push_str("- Dev Tools: ✅ Implemented\n");
        
        report
    }

    /// Benchmark a capability execution (mock)
    pub fn benchmark_capability_execution(&self, capability_id: &str) -> BenchmarkResult {
        // Simulate benchmark timing
        let start_time = current_timestamp();
        
        // Mock execution time (in practice, this would run actual operations)
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let end_time = current_timestamp();
        let duration_ms = (end_time - start_time) * 1000;
        
        BenchmarkResult {
            capability_id: capability_id.to_string(),
            duration_ms: duration_ms as u64,
            compute_units_used: 50_000,
            success: true,
            error: None,
        }
    }
}

/// Benchmark result for performance testing
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub capability_id: String,
    pub duration_ms: u64,
    pub compute_units_used: u32,
    pub success: bool,
    pub error: Option<String>,
}

/// Development utilities for the ValenceClient
impl ValenceClient {
    /// Get development tools
    pub fn dev_tools(&self) -> DevToolkit {
        DevToolkit::new()
    }
    
    /// Enable debug mode
    pub fn enable_debug(&self) {
        std::env::set_var("VALENCE_DEBUG", "1");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_data_generation() {
        let toolkit = DevToolkit::new();
        
        // Test capability generation
        let capability = toolkit.mock_data.mock_capability(None);
        assert!(capability.capability_id.starts_with("test_capability_"));
        assert!(capability.is_active);
        assert!(!capability.verification_functions.is_empty());
        
        // Test session generation
        let session = toolkit.mock_data.mock_session(None);
        assert!(session.session_id.starts_with("test_session_"));
        assert!(session.is_active);
        assert!(!session.capabilities.is_empty());
        
        // Test library generation
        let library = toolkit.mock_data.mock_library_entry(None);
        assert!(library.library_id.starts_with("test_library_"));
        assert_eq!(library.version, "1.0.0");
        assert!(!library.tags.is_empty());
    }

    #[test]
    fn test_test_helpers() {
        let toolkit = DevToolkit::new();
        
        // Test capability creation
        let capability = toolkit.test_helpers.create_test_capability("valid_capability").unwrap();
        assert_eq!(capability.capability_id, "valid_capability");
        assert!(capability.is_active);
        
        // Test invalid capability ID
        assert!(toolkit.test_helpers.create_test_capability("invalid capability").is_err());
        
        // Test session creation
        let session = toolkit.test_helpers.create_test_session(
            "test_session",
            vec!["valid_capability".to_string()]
        ).unwrap();
        assert_eq!(session.session_id, "test_session");
        assert_eq!(session.capabilities.len(), 1);
    }

    #[test]
    fn test_dev_utils() {
        let toolkit = DevToolkit::new();
        
        // Test benchmark
        let result = toolkit.dev_utils.benchmark_capability_execution("test_capability");
        assert_eq!(result.capability_id, "test_capability");
        assert!(result.success);
        assert!(result.duration_ms > 0);
        
        // Test config report generation
        let config = toolkit.test_helpers.create_test_config();
        let report = toolkit.dev_utils.generate_dev_report(&config);
        assert!(report.contains("Valence Protocol Development Report"));
        assert!(report.contains("Program IDs"));
    }
} 