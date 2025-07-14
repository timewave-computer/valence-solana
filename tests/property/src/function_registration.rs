use proptest::prelude::*;
use sha2::{Sha256, Digest};
use std::collections::{HashMap, HashSet};

/// Property: Function IDs must be deterministic
#[test]
fn prop_function_ids_are_deterministic() {
    proptest!(|(
        bytecode in prop::collection::vec(0u8..255, 100..1000),
        metadata in "[a-zA-Z0-9]{10,50}",
        iterations: u8,
    )| {
        let hash1 = compute_function_hash(&bytecode, &metadata);
        
        // Property: Same input always produces same hash
        for _ in 0..iterations {
            let hash2 = compute_function_hash(&bytecode, &metadata);
            prop_assert_eq!(
                hash1, hash2,
                "Function hash is not deterministic"
            );
        }
        
        // Property: Hash has expected length
        prop_assert_eq!(
            hash1.len(), 32,
            "Function hash has incorrect length"
        );
    });
}

/// Property: Function registration must be idempotent
#[test]
fn prop_function_registration_is_idempotent() {
    proptest!(|(
        function_id in any::<[u8; 32]>(),
        bytecode_hash in any::<[u8; 32]>(),
        registration_attempts in 1..10u8,
    )| {
        let mut registry = MockRegistry::new();
        
        // First registration
        let first_result = registry.register_function(function_id, bytecode_hash);
        prop_assert!(first_result.is_ok(), "First registration should succeed");
        
        // Property: Subsequent registrations should be idempotent
        for _ in 1..registration_attempts {
            let result = registry.register_function(function_id, bytecode_hash);
            prop_assert!(
                result.is_ok() || result == Err("Already registered"),
                "Unexpected registration result"
            );
            
            // State should remain consistent
            let registered = registry.get_function(function_id);
            prop_assert_eq!(
                registered, Some(bytecode_hash),
                "Function registration state inconsistent"
            );
        }
    });
}

/// Property: Function updates must maintain version history
#[test]
fn prop_function_updates_maintain_history() {
    proptest!(|(
        function_id in any::<[u8; 32]>(),
        versions in prop::collection::vec(any::<[u8; 32]>(), 1..20),
    )| {
        let mut registry = MockVersionedRegistry::new();
        let mut expected_versions = Vec::new();
        
        for (version_num, bytecode_hash) in versions.iter().enumerate() {
            registry.update_function(function_id, *bytecode_hash, version_num as u32);
            expected_versions.push(*bytecode_hash);
            
            // Property: All previous versions are maintained
            let history = registry.get_function_history(function_id);
            prop_assert_eq!(
                history.len(), version_num + 1,
                "Version history length mismatch"
            );
            
            // Property: Version order is preserved
            for (i, &hash) in expected_versions.iter().enumerate() {
                prop_assert_eq!(
                    history[i], hash,
                    "Version history order not preserved at index {}", i
                );
            }
            
            // Property: Latest version is current
            prop_assert_eq!(
                registry.get_current_version(function_id),
                Some(*bytecode_hash),
                "Current version mismatch"
            );
        }
    });
}

/// Property: Function capability requirements are enforced
#[test]
fn prop_function_capability_requirements_enforced() {
    proptest!(|(
        functions in prop::collection::vec(
            (any::<[u8; 32]>(), 0u64..u64::MAX),
            1..50
        ),
        session_caps in 0u64..u64::MAX,
    )| {
        let mut registry = MockRegistry::new();
        
        // Register functions with capability requirements
        for (func_id, required_caps) in &functions {
            registry.register_function_with_caps(*func_id, *required_caps);
        }
        
        // Property: Can only execute functions with sufficient capabilities
        for (func_id, required_caps) in &functions {
            let can_execute = (session_caps & required_caps) == *required_caps;
            
            prop_assert_eq!(
                registry.can_execute(*func_id, session_caps),
                can_execute,
                "Capability check incorrect for function"
            );
            
            // Property: Having more capabilities still allows execution
            if can_execute {
                prop_assert!(
                    registry.can_execute(*func_id, session_caps | 0xFF),
                    "Additional capabilities should not prevent execution"
                );
            }
        }
    });
}

/// Property: Function deregistration is atomic
#[test]
fn prop_function_deregistration_is_atomic() {
    proptest!(|(
        initial_functions in prop::collection::vec(
            any::<[u8; 32]>(),
            1..20
        ),
        deregister_indices in prop::collection::vec(any::<prop::sample::Index>(), 0..10),
    )| {
        let mut registry = MockRegistry::new();
        let mut active_functions: HashSet<[u8; 32]> = HashSet::new();
        
        // Register initial functions
        for func_id in &initial_functions {
            registry.register_function(*func_id, [0; 32]).unwrap();
            active_functions.insert(*func_id);
        }
        
        // Deregister functions
        for idx in deregister_indices {
            if !active_functions.is_empty() {
                let funcs: Vec<_> = active_functions.iter().cloned().collect();
                let func_to_remove = funcs[idx.index(funcs.len())];
                
                // Property: Deregistration is atomic
                let before_state = registry.get_function(func_to_remove);
                let result = registry.deregister_function(func_to_remove);
                let after_state = registry.get_function(func_to_remove);
                
                if result.is_ok() {
                    prop_assert!(before_state.is_some(), "Function should exist before deregistration");
                    prop_assert!(after_state.is_none(), "Function should not exist after deregistration");
                    active_functions.remove(&func_to_remove);
                }
                
                // Property: Other functions remain unchanged
                for &other_func in &active_functions {
                    prop_assert!(
                        registry.get_function(other_func).is_some(),
                        "Unrelated function was affected by deregistration"
                    );
                }
            }
        }
    });
}

/// Property: Function bytecode validation
#[test]
fn prop_function_bytecode_validation() {
    proptest!(|(
        bytecode in prop::collection::vec(0u8..255, 0..10000),
        corruption_indices in prop::collection::vec(any::<prop::sample::Index>(), 0..10),
    )| {
        let original_hash = compute_bytecode_hash(&bytecode);
        let mut corrupted = bytecode.clone();
        
        // Corrupt some bytes
        for idx in corruption_indices {
            if !corrupted.is_empty() {
                let index = idx.index(corrupted.len());
                corrupted[index] = corrupted[index].wrapping_add(1);
            }
        }
        
        if corrupted != bytecode {
            // Property: Corruption changes hash
            let corrupted_hash = compute_bytecode_hash(&corrupted);
            prop_assert_ne!(
                original_hash, corrupted_hash,
                "Bytecode corruption did not change hash"
            );
        }
        
        // Property: Validation detects invalid bytecode
        let is_valid = validate_bytecode(&bytecode);
        if !bytecode.is_empty() {
            // Simple validation: check for minimum size and magic bytes
            prop_assert_eq!(
                is_valid,
                bytecode.len() >= 8 && &bytecode[0..4] == b"FUNC",
                "Bytecode validation incorrect"
            );
        }
    });
}

/// Property: Function metadata constraints
#[test]
fn prop_function_metadata_constraints() {
    proptest!(|(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,63}",
        version in "[0-9]+\\.[0-9]+\\.[0-9]+",
        description in ".{0,1000}",
        size_limit: bool,
    )| {
        let metadata = FunctionMetadata {
            name: name.clone(),
            version: version.clone(),
            description: description.clone(),
        };
        
        // Property: Name constraints
        prop_assert!(
            metadata.name.len() <= 64,
            "Function name too long"
        );
        prop_assert!(
            metadata.name.chars().next().unwrap().is_alphabetic(),
            "Function name must start with letter"
        );
        
        // Property: Version format
        let version_parts: Vec<&str> = metadata.version.split('.').collect();
        prop_assert_eq!(
            version_parts.len(), 3,
            "Version must have three parts"
        );
        
        // Property: Total metadata size constraint
        let total_size = metadata.name.len() + metadata.version.len() + metadata.description.len();
        if size_limit {
            prop_assert!(
                total_size <= 2048,
                "Total metadata size exceeds limit"
            );
        }
    });
}

// Helper structures and functions
struct MockRegistry {
    functions: HashMap<[u8; 32], [u8; 32]>,
    capabilities: HashMap<[u8; 32], u64>,
}

impl MockRegistry {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
            capabilities: HashMap::new(),
        }
    }
    
    fn register_function(&mut self, id: [u8; 32], hash: [u8; 32]) -> Result<(), &'static str> {
        if self.functions.contains_key(&id) {
            Err("Already registered")
        } else {
            self.functions.insert(id, hash);
            Ok(())
        }
    }
    
    fn register_function_with_caps(&mut self, id: [u8; 32], caps: u64) {
        self.functions.insert(id, [0; 32]);
        self.capabilities.insert(id, caps);
    }
    
    fn get_function(&self, id: [u8; 32]) -> Option<[u8; 32]> {
        self.functions.get(&id).copied()
    }
    
    fn deregister_function(&mut self, id: [u8; 32]) -> Result<(), &'static str> {
        if self.functions.remove(&id).is_some() {
            self.capabilities.remove(&id);
            Ok(())
        } else {
            Err("Not found")
        }
    }
    
    fn can_execute(&self, id: [u8; 32], session_caps: u64) -> bool {
        if let Some(&required_caps) = self.capabilities.get(&id) {
            (session_caps & required_caps) == required_caps
        } else {
            false
        }
    }
}

struct MockVersionedRegistry {
    versions: HashMap<[u8; 32], Vec<[u8; 32]>>,
}

impl MockVersionedRegistry {
    fn new() -> Self {
        Self {
            versions: HashMap::new(),
        }
    }
    
    fn update_function(&mut self, id: [u8; 32], hash: [u8; 32], _version: u32) {
        self.versions.entry(id).or_insert_with(Vec::new).push(hash);
    }
    
    fn get_function_history(&self, id: [u8; 32]) -> Vec<[u8; 32]> {
        self.versions.get(&id).cloned().unwrap_or_default()
    }
    
    fn get_current_version(&self, id: [u8; 32]) -> Option<[u8; 32]> {
        self.versions.get(&id).and_then(|v| v.last()).copied()
    }
}

#[derive(Clone, Debug)]
struct FunctionMetadata {
    name: String,
    version: String,
    description: String,
}

fn compute_function_hash(bytecode: &[u8], metadata: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(bytecode);
    hasher.update(metadata.as_bytes());
    hasher.finalize().into()
}

fn compute_bytecode_hash(bytecode: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(bytecode);
    hasher.finalize().into()
}

fn validate_bytecode(bytecode: &[u8]) -> bool {
    bytecode.len() >= 8 && &bytecode[0..4] == b"FUNC"
}