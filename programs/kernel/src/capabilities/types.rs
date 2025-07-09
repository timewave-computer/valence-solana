/// Core capability type definitions for the Valence Protocol
use anchor_lang::prelude::*;

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
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum CapabilityScope {
    /// Global scope - accessible everywhere
    Global,
    /// Session scope - accessible within session
    Session,
    /// Namespace scope - accessible within specific namespace
    Namespace(String),
    /// Custom scope with specific rules
    Custom(String),
}

/// Capability type enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum CapabilityType {
    /// Function execution capability
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
    pub fn with_permission(mut self, permission: String) -> Self {
        self.required_permissions.push(permission);
        self
    }
    
    /// Set capability version
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

impl Default for CapabilityScope {
    fn default() -> Self {
        CapabilityScope::Session
    }
}

impl Default for CapabilityType {
    fn default() -> Self {
        CapabilityType::Function
    }
} 