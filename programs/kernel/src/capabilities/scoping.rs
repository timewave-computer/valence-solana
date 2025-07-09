/// Capability scoping and namespace access control
/// This module handles capability definitions, namespace management, and access validation
use anchor_lang::prelude::*;
use crate::sessions::isolation::{NamespaceCapability, ObjectId, FunctionInput};

// ======================= CORE CAPABILITY TYPES =======================

/// Capability definition structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CapabilityDefinition {
    /// Unique capability identifier
    pub id: String,
    /// Capability name
    pub name: String,
    /// Capability description
    pub description: String,
    /// Capability type
    pub capability_type: CapabilityType,
    /// Capability scope
    pub scope: CapabilityScope,
    /// Required permissions
    pub required_permissions: Vec<String>,
    /// Capability version
    pub version: String,
    /// Whether capability is active
    pub is_active: bool,
}

/// Capability scope enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Default)]
pub enum CapabilityScope {
    /// Global scope - accessible everywhere
    Global,
    /// Session scope - accessible within session
    #[default]
    Session,
    /// Namespace scope - accessible within specific namespace
    Namespace(String),
    /// Custom scope with specific rules
    Custom(String),
}

/// Capability type enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Default)]
pub enum CapabilityType {
    /// Function execution capability
    #[default]
    Function,
    /// Data access capability
    DataAccess,
    /// State modification capability
    StateModification,
    /// Verification capability
    Verification,
    /// Administrative capability
    Administrative,
    /// Composite capability (combines multiple capabilities)
    Composite,
}

impl CapabilityDefinition {
    /// Create a new capability definition
    pub fn new(
        id: String,
        name: String,
        description: String,
        capability_type: CapabilityType,
        scope: CapabilityScope,
    ) -> Self {
        Self {
            id,
            name,
            description,
            capability_type,
            scope,
            required_permissions: vec![],
            version: "1.0.0".to_string(),
            is_active: true,
        }
    }
    
    /// Add a required permission
    #[must_use]
    pub fn with_permission(mut self, permission: String) -> Self {
        self.required_permissions.push(permission);
        self
    }
    
    /// Set capability version
    #[must_use]
    pub fn with_version(mut self, version: String) -> Self {
        self.version = version;
        self
    }
    
    /// Check if capability matches given scope
    pub fn matches_scope(&self, scope: &CapabilityScope) -> bool {
        match (&self.scope, scope) {
            (CapabilityScope::Global, _) => true,
            (CapabilityScope::Session, CapabilityScope::Session) => true,
            (CapabilityScope::Namespace(a), CapabilityScope::Namespace(b)) => a == b,
            (CapabilityScope::Custom(a), CapabilityScope::Custom(b)) => a == b,
            _ => false,
        }
    }
    
    /// Check if capability is compatible with given type
    pub fn is_compatible_type(&self, capability_type: &CapabilityType) -> bool {
        &self.capability_type == capability_type || 
        self.capability_type == CapabilityType::Composite
    }
}

// ======================= NAMESPACE MANAGEMENT =======================

/// Namespace access control and validation
pub struct NamespaceManager;

impl NamespaceManager {
    /// Check if a session has access to a namespace
    pub fn check_namespace_access(
        session: &Pubkey,
        namespace: &str,
        _capabilities: &[String],
    ) -> Result<bool> {
        // Implementation will be added as we consolidate functionality
        msg!("Checking namespace access for session: {} in namespace: {}", session, namespace);
        Ok(true)
    }
    
    /// Validate namespace permissions for a capability
    pub fn validate_capability_namespace(
        capability_id: &str,
        namespace: &str,
        allowed_namespaces: &[String],
    ) -> Result<bool> {
        // Implementation will be added as we consolidate functionality
        msg!("Validating capability: {} in namespace: {}", capability_id, namespace);
        Ok(allowed_namespaces.contains(&namespace.to_string()))
    }
    
    /// Get all accessible namespaces for a session
    pub fn get_accessible_namespaces(
        session: &Pubkey,
        _capabilities: &[String],
    ) -> Result<Vec<String>> {
        // Implementation will be added as we consolidate functionality
        msg!("Getting accessible namespaces for session: {}", session);
        Ok(vec!["default".to_string()])
    }
}

/// Namespace access patterns
#[derive(Debug, Clone, Default)]
pub enum NamespaceAccess {
    /// Full access to all namespaces
    Global,
    /// Access to specific namespaces
    Scoped(Vec<String>),
    /// No namespace access
    #[default]
    None,
}

// ======================= VALIDATION AND ACCOUNT CONTEXTS =======================

/// Pure namespace scope validation (no accounts needed)
#[derive(Accounts)]
pub struct ValidateNamespaceScope<'info> {
    /// Any signer can call this pure verification function
    pub caller: Signer<'info>,
}

/// Pure object access check (no accounts needed)
#[derive(Accounts)]
pub struct CheckObjectAccess<'info> {
    /// Any signer can call this pure verification function
    pub caller: Signer<'info>,
}

/// Verify composition of multiple capabilities (no accounts needed)
#[derive(Accounts)]
pub struct VerifyCapabilityComposition<'info> {
    /// Any signer can call this pure verification function
    pub caller: Signer<'info>,
}

/// Result of namespace capability validation
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CapabilityValidationResult {
    /// Whether the capability is valid
    pub is_valid: bool,
    /// Whether access is granted
    pub access_granted: bool,
    /// Reason for the decision
    pub reason: String,
    /// Objects that were validated
    pub validated_objects: Vec<ObjectId>,
    /// Namespace ID that was checked
    pub namespace_id: String,
}

impl CapabilityValidationResult {
    pub fn granted(namespace_id: String, validated_objects: Vec<ObjectId>, reason: String) -> Self {
        Self {
            is_valid: true,
            access_granted: true,
            reason,
            validated_objects,
            namespace_id,
        }
    }
    
    pub fn denied(namespace_id: String, validated_objects: Vec<ObjectId>, reason: String) -> Self {
        Self {
            is_valid: true,
            access_granted: false,
            reason,
            validated_objects,
            namespace_id,
        }
    }
    
    pub fn invalid(reason: String) -> Self {
        Self {
            is_valid: false,
            access_granted: false,
            reason,
            validated_objects: vec![],
            namespace_id: String::new(),
        }
    }
}

// ======================= NAMESPACE CAPABILITY EXTENSIONS =======================

/// Helper functions for capability-based namespace operations
impl NamespaceCapability {
    /// Validate a function call respects namespace boundaries
    pub fn validate_function_call(
        &self,
        _function_input: &FunctionInput,
        accessed_objects: &[ObjectId],
    ) -> CapabilityValidationResult {
        // Verify capability integrity first
        if !self.verify_integrity() {
            return CapabilityValidationResult::invalid(
                "Capability hash verification failed".to_string()
            );
        }
        
        // Check object count limit
        if accessed_objects.len() > self.max_objects_per_call as usize {
            return CapabilityValidationResult::denied(
                self.namespace_id.clone(),
                accessed_objects.to_vec(),
                format!("Too many objects accessed: {} > {}", accessed_objects.len(), self.max_objects_per_call)
            );
        }
        
        // Check each object is within namespace
        for obj in accessed_objects {
            if !self.can_access_object(obj) {
                return CapabilityValidationResult::denied(
                    self.namespace_id.clone(),
                    accessed_objects.to_vec(),
                    format!("Object not in namespace: {obj:?}")
                );
            }
            
            if !self.is_object_type_allowed(&obj.object_type) {
                return CapabilityValidationResult::denied(
                    self.namespace_id.clone(),
                    accessed_objects.to_vec(),
                    format!("Object type not allowed: {}", obj.object_type)
                );
            }
        }
        
        CapabilityValidationResult::granted(
            self.namespace_id.clone(),
            accessed_objects.to_vec(),
            "All objects within namespace capability".to_string()
        )
    }
    
    /// Validate function input for execution
    pub fn validate_function_input(
        &self,
        _function_input: &FunctionInput,
    ) -> Result<()> {
        // TODO: Implement function input validation
        // This would validate the input parameters against the capability's requirements
        Ok(())
    }
    
    /// Create union of two capabilities (objects accessible in either)
    #[must_use]
    pub fn union(&self, other: &NamespaceCapability) -> NamespaceCapability {
        let mut union_objects = self.accessible_objects.clone();
        for obj in &other.accessible_objects {
            if !union_objects.contains(obj) {
                union_objects.push(obj.clone());
            }
        }
        
        let mut union_types = self.allowed_object_types.clone();
        for obj_type in &other.allowed_object_types {
            if !union_types.contains(obj_type) {
                union_types.push(obj_type.clone());
            }
        }
        
        NamespaceCapability::new(
            format!("{}∪{}", self.namespace_id, other.namespace_id),
            union_objects,
            union_types,
            self.max_objects_per_call.max(other.max_objects_per_call),
        )
    }
} 