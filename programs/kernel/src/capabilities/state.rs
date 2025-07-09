// Core capability state management
use anchor_lang::prelude::*;
use crate::sessions::isolation::{NamespaceCapability, ObjectId, FunctionInput};

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
                    format!("Object not in namespace: {:?}", obj)
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