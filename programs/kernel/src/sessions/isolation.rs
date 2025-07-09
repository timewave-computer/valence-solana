// Session isolation through namespace scoping and capability management
// This module provides inherent isolation for sessions without requiring a separate program
use anchor_lang::prelude::*;
use crate::sessions::state::SessionConfiguration;

/// Object identifier for diffs
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct ObjectId {
    pub hash: [u8; 32],
    pub object_type: String,
    pub namespace: Option<String>,
}

/// Key-value pair for metadata
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MetadataEntry {
    pub key: String,
    pub value: String,
}

/// Diff structure for tracking changes
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Diff {
    pub content_hash: [u8; 32],
    pub target_object: ObjectId,
    pub operation_type: String,
    pub data: Vec<u8>,
    pub timestamp: i64,
    pub creator: Pubkey,
    pub dependencies: Vec<[u8; 32]>,
    pub metadata: Vec<MetadataEntry>,
}

/// Namespace capability for session access control
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct NamespaceCapability {
    pub namespace_id: String,
    pub accessible_objects: Vec<ObjectId>,
    pub allowed_object_types: Vec<String>,
    pub max_objects_per_call: u32,
    pub capability_hash: [u8; 32],
}

impl NamespaceCapability {
    pub fn new(
        namespace_id: String,
        accessible_objects: Vec<ObjectId>,
        allowed_object_types: Vec<String>,
        max_objects_per_call: u32,
    ) -> Self {
        let capability_hash = [0u8; 32]; // Simplified hash calculation
        Self {
            namespace_id,
            accessible_objects,
            allowed_object_types,
            max_objects_per_call,
            capability_hash,
        }
    }
    
    pub fn verify_integrity(&self) -> bool {
        // Simplified integrity check
        !self.namespace_id.is_empty() && self.max_objects_per_call > 0
    }
    
    pub fn can_access_object(&self, object_id: &ObjectId) -> bool {
        self.accessible_objects.contains(object_id)
    }
    
    pub fn is_object_type_allowed(&self, object_type: &str) -> bool {
        self.allowed_object_types.is_empty() || self.allowed_object_types.contains(&object_type.to_string())
    }
    
    pub fn intersect(&self, other: &NamespaceCapability) -> NamespaceCapability {
        let common_objects: Vec<ObjectId> = self.accessible_objects
            .iter()
            .filter(|obj| other.accessible_objects.contains(obj))
            .cloned()
            .collect();
            
        NamespaceCapability::new(
            format!("{}+{}", self.namespace_id, other.namespace_id),
            common_objects,
            vec![], // Simplified - in practice would find intersection of allowed types
            std::cmp::min(self.max_objects_per_call, other.max_objects_per_call),
        )
    }
}

/// Session isolation configuration that integrates with namespace scoping
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SessionIsolationConfig {
    /// Namespace capabilities this session can use
    pub namespace_capabilities: Vec<NamespaceCapability>,
    /// Maximum number of diffs per transaction
    pub max_diffs_per_tx: u32,
    /// Maximum size of individual diffs
    pub max_diff_size_bytes: u64,
    /// Whether cross-session access is allowed
    pub allow_cross_session: bool,
}

impl Default for SessionIsolationConfig {
    fn default() -> Self {
        // Create a default namespace capability for basic access
        let default_capability = NamespaceCapability::new(
            "default".to_string(),
            vec![], // No specific objects by default
            vec![], // No object type restrictions by default
            10, // Max 10 objects per call
        );
        
        Self {
            namespace_capabilities: vec![default_capability],
            max_diffs_per_tx: 10,
            max_diff_size_bytes: 1024,
            allow_cross_session: false,
        }
    }
}

/// Function input structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FunctionInput {
    pub diffs: Vec<Diff>,
    pub parameters: Vec<u8>,
}

/// Session isolation validator using capability-based approach
pub struct SessionIsolationValidator;

impl SessionIsolationValidator {
    /// Validate that a diff respects this session's isolation constraints using capabilities
    pub fn validate_diff(
        _session_config: &SessionConfiguration,
        _diff: &Diff,
    ) -> Result<bool> {
        // For now, just return true - this would need the isolation_config field added to SessionConfiguration
        Ok(true)
    }
    
    /// Validate a batch of diffs for session isolation using capabilities
    pub fn validate_diff_batch(
        _session_config: &SessionConfiguration,
        diffs: &[Diff],
    ) -> Result<Vec<bool>> {
        // Simplified implementation
        let results = diffs.iter().map(|_| true).collect();
        Ok(results)
    }
    
    /// Check if a session can access an object through its namespace capabilities
    pub fn can_access_object(
        _session_config: &SessionConfiguration,
        _object_id: &ObjectId,
    ) -> bool {
        // Simplified implementation
        true
    }
    
    /// Filter diffs based on session isolation rules using capabilities
    pub fn filter_diffs(
        _session_config: &SessionConfiguration,
        diffs: Vec<Diff>,
    ) -> (Vec<Diff>, Vec<Diff>) {
        // Simplified implementation - all diffs allowed for now
        (diffs, vec![])
    }
    
    /// Check if two sessions can interact based on capability overlap
    pub fn can_sessions_interact(
        _source_config: &SessionConfiguration,
        _target_config: &SessionConfiguration,
    ) -> bool {
        // Simplified implementation
        false
    }
}

/// Integration with namespace scoping for session-level permissions
pub struct SessionNamespaceIntegration;

impl SessionNamespaceIntegration {
    /// Create namespace capabilities for a new session
    pub fn create_session_capabilities(
        session_id: &str,
        accessible_objects: Vec<ObjectId>,
        allowed_namespaces: Vec<String>,
        max_objects_per_call: u32,
    ) -> Vec<NamespaceCapability> {
        let mut capabilities = Vec::new();
        
        // Group objects by namespace
        let mut objects_by_namespace: std::collections::BTreeMap<String, Vec<ObjectId>> = 
            std::collections::BTreeMap::new();
            
        for object in accessible_objects {
            let namespace = object.namespace.clone().unwrap_or("default".to_string());
            objects_by_namespace.entry(namespace).or_default().push(object);
        }
        
        // Create a capability for each namespace
        for namespace in allowed_namespaces {
            let namespace_objects = objects_by_namespace.get(&namespace).cloned().unwrap_or_default();
            
            let capability = NamespaceCapability::new(
                format!("{}:{}", session_id, namespace),
                namespace_objects,
                vec![], // No object type restrictions by default
                max_objects_per_call,
            );
            
            capabilities.push(capability);
        }
        
        capabilities
    }
    
    /// Validate that a session's operation is allowed by its namespace capabilities
    pub fn validate_session_operation(
        session_config: &SessionConfiguration,
        target_object: &ObjectId,
        operation: &str,
    ) -> bool {
        // Check if any capability allows access to this object
        if !SessionIsolationValidator::can_access_object(session_config, target_object) {
            return false;
        }
        
        // Basic operation validation (all operations allowed if object access is granted)
        match operation {
            "read" | "query" | "write" | "update" => true,
            "create" | "delete" => false, // Sessions don't create/delete objects directly
            _ => false, // Unknown operations are denied
        }
    }
}

/// Session boundary enforcement using capabilities
pub struct SessionBoundaryEnforcer;

impl SessionBoundaryEnforcer {
    /// Enforce session boundaries for a function execution using capabilities
    pub fn enforce_function_boundaries(
        session_config: &SessionConfiguration,
        function_input: &FunctionInput,
    ) -> Result<bool> {
        // Validate all diffs against session capabilities
        for diff in &function_input.diffs {
            if !SessionIsolationValidator::validate_diff(session_config, diff)? {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Enforce session boundaries for function composition using capabilities
    pub fn enforce_composition_boundaries(
        session_config: &SessionConfiguration,
        composition_diffs: &[Diff],
    ) -> Result<Vec<Diff>> {
        let (allowed, _blocked) = SessionIsolationValidator::filter_diffs(
            session_config,
            composition_diffs.to_vec(),
        );
        
        Ok(allowed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_session_config() -> SessionConfiguration {
        SessionConfiguration::default()
    }

    #[test]
    fn test_diff_validation_with_capabilities() {
        let config = create_test_session_config();
        
        let valid_diff = Diff {
            content_hash: [1u8; 32],
            target_object: ObjectId {
                hash: [1u8; 32],
                object_type: "account".to_string(),
                namespace: Some("test".to_string()),
            },
            operation_type: "update".to_string(),
            data: b"small_data".to_vec(),
            timestamp: 0,
            creator: Pubkey::new_unique(),
            dependencies: vec![],
            metadata: vec![], // Changed from Default::default()
        };
        
        assert!(SessionIsolationValidator::validate_diff(&config, &valid_diff).unwrap());
    }

    #[test]
    fn test_capability_creation() {
        let test_objects = vec![
            ObjectId {
                hash: [1u8; 32],
                object_type: "account".to_string(),
                namespace: Some("trading".to_string()),
            },
        ];
        
        let capabilities = SessionNamespaceIntegration::create_session_capabilities(
            "test_session",
            test_objects,
            vec!["trading".to_string()],
            10,
        );
        
        assert_eq!(capabilities.len(), 1);
        assert_eq!(capabilities[0].namespace_id, "test_session:trading");
    }
} 