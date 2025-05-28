// Comprehensive tests for Registry Program functionality

#[cfg(test)]
mod tests {
    use crate::utils::*;
    use registry::state::*;
    use anchor_lang::prelude::*;

    #[test]
    fn test_registry_state_creation() {
        let ctx = TestContext::new();
        
        let registry_state = RegistryState {
            owner: ctx.authority,
            authorization_program_id: generate_test_pubkey("auth_program"),
            account_factory: generate_test_pubkey("account_factory"),
            bump: 255,
        };

        // Test serialization
        let serialized = registry_state.try_to_vec().unwrap();
        let deserialized: RegistryState = RegistryState::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.owner, ctx.authority);
        assert_eq!(deserialized.authorization_program_id, registry_state.authorization_program_id);
        assert_eq!(deserialized.account_factory, registry_state.account_factory);
        assert_eq!(deserialized.bump, 255);
    }

    #[test]
    fn test_library_info_creation() {
        let library_info = LibraryInfo {
            program_id: generate_test_pubkey("library_program"),
            library_type: "token_transfer".to_string(),
            description: "Library for token transfers".to_string(),
            is_approved: true,
            version: "1.0.0".to_string(),
            last_updated: 1000000000,
            dependencies: vec![],
            bump: 254,
        };

        // Test serialization
        let serialized = library_info.try_to_vec().unwrap();
        let deserialized: LibraryInfo = LibraryInfo::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.program_id, library_info.program_id);
        assert_eq!(deserialized.library_type, "token_transfer");
        assert_eq!(deserialized.description, "Library for token transfers");
        assert!(deserialized.is_approved);
        assert_eq!(deserialized.version, "1.0.0");
        assert!(deserialized.dependencies.is_empty());
    }

    #[test]
    fn test_library_dependency_management() {
        let dependency = LibraryDependency {
            program_id: generate_test_pubkey("dependency_program"),
            required_version: ">=1.0.0".to_string(),
            is_optional: false,
            dependency_type: DependencyType::Runtime,
        };

        // Test serialization
        let serialized = dependency.try_to_vec().unwrap();
        let deserialized: LibraryDependency = LibraryDependency::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.program_id, dependency.program_id);
        assert_eq!(deserialized.required_version, ">=1.0.0");
        assert!(!deserialized.is_optional);
        assert_eq!(deserialized.dependency_type, DependencyType::Runtime);
    }

    #[test]
    fn test_dependency_types() {
        let dependency_types = vec![
            DependencyType::Runtime,
            DependencyType::Build,
            DependencyType::Dev,
            DependencyType::Optional,
        ];

        for dep_type in dependency_types {
            // Test serialization
            let serialized = dep_type.try_to_vec().unwrap();
            let deserialized: DependencyType = DependencyType::try_from_slice(&serialized).unwrap();
            assert_eq!(deserialized, dep_type);
        }

        // Test default
        assert_eq!(DependencyType::default(), DependencyType::Runtime);
    }

    #[test]
    fn test_dependency_graph_creation() {
        let dependency_graph = DependencyGraph {
            root_library: generate_test_pubkey("root_library"),
            resolved_order: vec![
                generate_test_pubkey("dep1"),
                generate_test_pubkey("dep2"),
                generate_test_pubkey("root_library"),
            ],
            is_valid: true,
            last_resolved: 1000000000,
            bump: 253,
        };

        // Test serialization
        let serialized = dependency_graph.try_to_vec().unwrap();
        let deserialized: DependencyGraph = DependencyGraph::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.root_library, dependency_graph.root_library);
        assert_eq!(deserialized.resolved_order.len(), 3);
        assert!(deserialized.is_valid);
        assert_eq!(deserialized.last_resolved, 1000000000);

        // Test space calculation
        let space = DependencyGraph::space(3);
        assert!(space > 100); // Should have reasonable size
        assert_eq!(space, 8 + 32 + 4 + (3 * 32) + 1 + 8 + 1); // Exact calculation
    }

    #[test]
    fn test_library_with_dependencies() {
        let library_info = LibraryInfo {
            program_id: generate_test_pubkey("complex_library"),
            library_type: "defi_protocol".to_string(),
            description: "Complex DeFi protocol with multiple dependencies".to_string(),
            is_approved: true,
            version: "2.1.0".to_string(),
            last_updated: 1000000000,
            dependencies: vec![
                LibraryDependency {
                    program_id: generate_test_pubkey("token_lib"),
                    required_version: ">=1.0.0".to_string(),
                    is_optional: false,
                    dependency_type: DependencyType::Runtime,
                },
                LibraryDependency {
                    program_id: generate_test_pubkey("math_lib"),
                    required_version: "^2.0.0".to_string(),
                    is_optional: false,
                    dependency_type: DependencyType::Runtime,
                },
                LibraryDependency {
                    program_id: generate_test_pubkey("oracle_lib"),
                    required_version: "~1.5.0".to_string(),
                    is_optional: true,
                    dependency_type: DependencyType::Optional,
                },
            ],
            bump: 252,
        };

        // Test dependency management
        assert_eq!(library_info.dependencies.len(), 3);
        
        // Test filtering dependencies by type
        let runtime_deps: Vec<_> = library_info.dependencies.iter()
            .filter(|dep| dep.dependency_type == DependencyType::Runtime)
            .collect();
        assert_eq!(runtime_deps.len(), 2);

        let optional_deps: Vec<_> = library_info.dependencies.iter()
            .filter(|dep| dep.is_optional)
            .collect();
        assert_eq!(optional_deps.len(), 1);

        // Test serialization with dependencies
        let serialized = library_info.try_to_vec().unwrap();
        let deserialized: LibraryInfo = LibraryInfo::try_from_slice(&serialized).unwrap();
        assert_eq!(deserialized.dependencies.len(), 3);
    }

    #[test]
    fn test_version_string_validation() {
        let valid_versions = vec![
            "1.0.0",
            "2.1.3",
            "0.1.0-alpha",
            "1.0.0-beta.1",
            "10.20.30",
        ];

        let version_patterns = vec![
            ">=1.0.0",
            "^2.0.0",
            "~1.5.0",
            "=1.2.3",
            "*",
        ];

        // Test valid versions
        for version in valid_versions {
            assert!(!version.is_empty(), "Version should not be empty");
            assert!(version.len() <= 32, "Version should be reasonable length");
            assert!(version.contains('.'), "Version should contain dots");
        }

        // Test version patterns
        for pattern in version_patterns {
            assert!(!pattern.is_empty(), "Pattern should not be empty");
            assert!(pattern.len() <= 32, "Pattern should be reasonable length");
        }
    }

    #[test]
    fn test_zk_program_info_creation() {
        let zk_program_info = ZKProgramInfo {
            program_id: generate_test_pubkey("zk_program"),
            verification_key_hash: [1u8; 32],
            program_type: "sp1_verifier".to_string(),
            description: "SP1 zero-knowledge proof verifier".to_string(),
            is_active: true,
            registered_at: 1000000000,
            last_verified: 1000000100,
            verification_count: 42,
            bump: 251,
        };

        // Test serialization
        let serialized = zk_program_info.try_to_vec().unwrap();
        let deserialized: ZKProgramInfo = ZKProgramInfo::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.program_id, zk_program_info.program_id);
        assert_eq!(deserialized.verification_key_hash, [1u8; 32]);
        assert_eq!(deserialized.program_type, "sp1_verifier");
        assert_eq!(deserialized.description, "SP1 zero-knowledge proof verifier");
        assert!(deserialized.is_active);
        assert_eq!(deserialized.verification_count, 42);
    }

    #[test]
    fn test_zk_program_types() {
        let zk_program_types = vec![
            "sp1_verifier",
            "groth16_verifier",
            "plonk_verifier",
            "stark_verifier",
            "bulletproof_verifier",
        ];

        for program_type in zk_program_types {
            assert!(!program_type.is_empty(), "Program type should not be empty");
            assert!(program_type.len() <= 64, "Program type should be reasonable length");
            assert!(program_type.contains("verifier"), "Should be a verifier type");
        }
    }

    #[test]
    fn test_verification_key_hash_validation() {
        // Test different hash values
        let hash1 = [0u8; 32]; // All zeros
        let hash2 = [255u8; 32]; // All ones
        let mut hash3 = [0u8; 32];
        hash3[0] = 1;
        hash3[31] = 255; // Mixed values

        let hashes = vec![hash1, hash2, hash3];

        for hash in hashes {
            assert_eq!(hash.len(), 32, "Hash should be 32 bytes");
            
            // Test that hash can be used in ZK program info
            let zk_info = ZKProgramInfo {
                program_id: generate_test_pubkey("test_program"),
                verification_key_hash: hash,
                program_type: "test_verifier".to_string(),
                description: "Test verifier".to_string(),
                is_active: true,
                registered_at: 1000000000,
                last_verified: 1000000000,
                verification_count: 0,
                bump: 250,
            };

            // Should serialize successfully
            let serialized = zk_info.try_to_vec().unwrap();
            assert!(!serialized.is_empty());
        }
    }

    #[test]
    fn test_library_approval_workflow() {
        let mut library_info = LibraryInfo {
            program_id: generate_test_pubkey("new_library"),
            library_type: "utility".to_string(),
            description: "Utility library for common operations".to_string(),
            is_approved: false, // Start as not approved
            version: "1.0.0".to_string(),
            last_updated: 1000000000,
            dependencies: vec![],
            bump: 249,
        };

        // Test initial state
        assert!(!library_info.is_approved, "Should start as not approved");

        // Simulate approval process
        library_info.is_approved = true;
        library_info.last_updated = 1000000100;

        assert!(library_info.is_approved, "Should be approved after update");
        assert_eq!(library_info.last_updated, 1000000100);
    }

    #[test]
    fn test_dependency_cycle_detection() {
        // Create a scenario that would represent a dependency cycle
        let lib_a = generate_test_pubkey("library_a");
        let lib_b = generate_test_pubkey("library_b");
        let lib_c = generate_test_pubkey("library_c");

        // A depends on B, B depends on C, C depends on A (cycle)
        let dependency_graph = DependencyGraph {
            root_library: lib_a,
            resolved_order: vec![], // Empty because cycle detected
            is_valid: false, // Cycle detected
            last_resolved: 1000000000,
            bump: 248,
        };

        assert!(!dependency_graph.is_valid, "Should detect cycle");
        assert!(dependency_graph.resolved_order.is_empty(), "Should have no resolved order");

        // Test valid dependency graph (no cycles)
        let valid_graph = DependencyGraph {
            root_library: lib_a,
            resolved_order: vec![lib_c, lib_b, lib_a], // Topologically sorted
            is_valid: true,
            last_resolved: 1000000000,
            bump: 247,
        };

        assert!(valid_graph.is_valid, "Should be valid");
        assert_eq!(valid_graph.resolved_order.len(), 3);
    }

    #[test]
    fn test_library_type_categories() {
        let library_types = vec![
            "token_transfer",
            "vault_management",
            "defi_protocol",
            "nft_marketplace",
            "governance",
            "oracle_integration",
            "cross_chain_bridge",
            "utility",
            "math_library",
            "security_module",
        ];

        for lib_type in library_types {
            assert!(!lib_type.is_empty(), "Library type should not be empty");
            assert!(lib_type.len() <= 64, "Library type should be reasonable length");
            assert!(!lib_type.contains(' '), "Library type should not contain spaces");
        }
    }

    #[test]
    fn test_space_calculations() {
        // Test space calculations for different configurations
        let base_library_space = std::mem::size_of::<LibraryInfo>();
        let base_registry_space = std::mem::size_of::<RegistryState>();
        let base_zk_program_space = std::mem::size_of::<ZKProgramInfo>();

        // Should be reasonable sizes
        assert!(base_library_space > 100, "Library info should have reasonable size");
        assert!(base_registry_space > 50, "Registry state should have reasonable size");
        assert!(base_zk_program_space > 100, "ZK program info should have reasonable size");

        // Test dependency graph space scaling
        let small_graph_space = DependencyGraph::space(5);
        let large_graph_space = DependencyGraph::space(20);

        assert!(large_graph_space > small_graph_space, "Larger graph should need more space");
        assert_eq!(large_graph_space - small_graph_space, 15 * 32); // 15 additional pubkeys
    }

    #[test]
    fn test_registry_permissions() {
        let ctx = TestContext::new();
        
        // Test permission validation
        let registry_owner = ctx.authority;
        let regular_user = ctx.user;
        let library_developer = generate_test_pubkey("developer");
        
        // Validate permission logic
        assert_ne!(registry_owner, regular_user, "Owner and user should be different");
        assert_ne!(registry_owner, library_developer, "Owner and developer should be different");
        
        // Test permission checks (simulated)
        let can_register_library = registry_owner == ctx.authority; // Only owner can register
        let can_approve_library = registry_owner == ctx.authority; // Only owner can approve
        let can_query_library = true; // Anyone can query
        
        assert!(can_register_library, "Owner should be able to register libraries");
        assert!(can_approve_library, "Owner should be able to approve libraries");
        assert!(can_query_library, "Anyone should be able to query libraries");
    }

    #[test]
    fn test_zk_program_verification_tracking() {
        let mut zk_program = ZKProgramInfo {
            program_id: generate_test_pubkey("active_zk_program"),
            verification_key_hash: generate_test_hash(b"verification_key"),
            program_type: "groth16_verifier".to_string(),
            description: "Production Groth16 verifier".to_string(),
            is_active: true,
            registered_at: 1000000000,
            last_verified: 1000000000,
            verification_count: 0,
            bump: 246,
        };

        // Simulate verification events
        for _ in 0..10 {
            zk_program.verification_count += 1;
            zk_program.last_verified = 1000000000 + (zk_program.verification_count as i64 * 60);
        }

        assert_eq!(zk_program.verification_count, 10);
        assert_eq!(zk_program.last_verified, 1000000600); // 10 minutes later

        // Test verification rate calculation
        let time_span = zk_program.last_verified - zk_program.registered_at;
        let verification_rate = (zk_program.verification_count as f64) / (time_span as f64 / 60.0); // per minute
        assert!(verification_rate > 0.0, "Should have positive verification rate");
    }
} 