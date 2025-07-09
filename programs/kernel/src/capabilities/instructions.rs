// Capability-based namespace scoping for Valence Protocol
// Namespaces are capability maps that constrain object access in functions
use anchor_lang::prelude::*;
use crate::capabilities::{ShardState, ExecutionContext, EvalConfig, PartialOrder};
use crate::error::NamespaceScopingError;

/// Simple object identifier for namespace scoping
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct ObjectId {
    pub id: [u8; 32],
    pub object_type: String,
}

/// Simple diff structure for object changes
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Diff {
    pub target_object: ObjectId,
    pub operation: String,
    pub data: Vec<u8>,
}

/// Function input structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FunctionInput {
    pub data: Vec<u8>,
    pub version: String,
}

/// A namespace capability that constrains object access
/// This is passed as a parameter to functions, not stored as mutable state
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct NamespaceCapability {
    /// Unique identifier for this namespace
    pub namespace_id: String,
    /// Objects that can be accessed within this namespace
    pub accessible_objects: Vec<ObjectId>,
    /// Object type constraints (empty = no restrictions)
    pub allowed_object_types: Vec<String>,
    /// Maximum objects that can be accessed in a single function call
    pub max_objects_per_call: u32,
    /// Hash for integrity verification
    pub capability_hash: [u8; 32],
}

impl NamespaceCapability {
    /// Create a new namespace capability
    pub fn new(
        namespace_id: String,
        accessible_objects: Vec<ObjectId>,
        allowed_object_types: Vec<String>,
        max_objects_per_call: u32,
    ) -> Self {
        let mut capability = Self {
            namespace_id,
            accessible_objects,
            allowed_object_types,
            max_objects_per_call,
            capability_hash: [0u8; 32],
        };
        
        // Calculate integrity hash
        capability.capability_hash = capability.calculate_hash();
        capability
    }
    
    /// Calculate hash for capability integrity
    pub fn calculate_hash(&self) -> [u8; 32] {
        use anchor_lang::solana_program::hash::hash;
        
        let mut data = Vec::new();
        data.extend_from_slice(self.namespace_id.as_bytes());
        
        // Serialize accessible objects
        for obj in &self.accessible_objects {
            data.extend_from_slice(&obj.id);
            data.extend_from_slice(obj.object_type.as_bytes());
        }
        
        // Serialize allowed types
        for obj_type in &self.allowed_object_types {
            data.extend_from_slice(obj_type.as_bytes());
        }
        
        data.extend_from_slice(&self.max_objects_per_call.to_le_bytes());
        
        hash(&data).to_bytes()
    }
    
    /// Verify capability integrity
    pub fn verify_integrity(&self) -> bool {
        let expected_hash = self.calculate_hash();
        self.capability_hash == expected_hash
    }
    
    /// Check if an object is accessible within this namespace
    pub fn can_access_object(&self, object_id: &ObjectId) -> bool {
        self.accessible_objects.iter().any(|obj| obj == object_id)
    }
    
    /// Check if object type is allowed
    pub fn is_object_type_allowed(&self, object_type: &str) -> bool {
        if self.allowed_object_types.is_empty() {
            true // No restrictions
        } else {
            self.allowed_object_types.iter().any(|allowed| allowed == object_type)
        }
    }
    
    /// Validate that proposed diffs respect namespace boundaries
    pub fn validate_diffs(&self, proposed_diffs: &[Diff]) -> Result<()> {
        // Check object count limit
        require!(
            proposed_diffs.len() <= self.max_objects_per_call as usize,
            NamespaceScopingError::NamespaceTooManyObjects
        );
        
        // Check each diff target is within namespace
        for diff in proposed_diffs {
            // Verify object is accessible
            require!(
                self.can_access_object(&diff.target_object),
                NamespaceScopingError::NamespaceObjectNotInNamespace
            );
            
            // Verify object type is allowed
            require!(
                self.is_object_type_allowed(&diff.target_object.object_type),
                NamespaceScopingError::NamespaceObjectTypeNotAllowed
            );
        }
        
        Ok(())
    }
    
    /// Create a sub-capability with restricted access
    pub fn create_sub_capability(
        &self,
        sub_namespace_id: String,
        restricted_objects: Vec<ObjectId>,
    ) -> Result<NamespaceCapability> {
        // Verify all restricted objects are within parent capability
        for obj in &restricted_objects {
            require!(
                self.can_access_object(obj),
                NamespaceScopingError::NamespaceObjectNotInNamespace
            );
        }
        
        // Capture length before moving restricted_objects
        let restricted_objects_len = restricted_objects.len() as u32;
        
        Ok(NamespaceCapability::new(
            sub_namespace_id,
            restricted_objects,
            self.allowed_object_types.clone(),
            self.max_objects_per_call.min(restricted_objects_len),
        ))
    }
}

// ========== EVAL OPERATION INSTRUCTIONS ==========

/// Initialize shard with eval configuration
#[derive(Accounts)]
pub struct InitializeShard<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = ShardState::SPACE,
        seeds = [b"shard_state"],
        bump
    )]
    pub shard_state: Account<'info, ShardState>,
    
    pub system_program: Program<'info, System>,
}

/// Execute capability with embedded eval logic
#[derive(Accounts)]
pub struct ExecuteCapability<'info> {
    #[account(mut)]
    pub shard_state: Account<'info, ShardState>,
    
    pub caller: Signer<'info>,
}

/// Update eval configuration
#[derive(Accounts)]
pub struct UpdateEvalConfig<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"shard_state"],
        bump = shard_state.bump
    )]
    pub shard_state: Account<'info, ShardState>,
    
    pub authority: Signer<'info>,
}

/// Set ordering rules for the shard
#[derive(Accounts)]
pub struct SetOrderingRules<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"shard_state"],
        bump = shard_state.bump
    )]
    pub shard_state: Account<'info, ShardState>,
    
    pub authority: Signer<'info>,
}

/// Initialize shard with eval capabilities
pub fn initialize_shard(
    ctx: Context<InitializeShard>,
    processor_program: Pubkey,
) -> Result<()> {
    let shard_state = &mut ctx.accounts.shard_state;
    shard_state.process_initialize(
        ctx.accounts.authority.key(),
        processor_program,
        ctx.bumps.shard_state,
    )?;
    
    msg!("Shard initialized with eval capabilities");
    Ok(())
}

/// Execute capability using embedded eval logic
pub fn execute_capability(
    ctx: Context<ExecuteCapability>,
    capability_id: String,
    input_data: Vec<u8>,
) -> Result<()> {
    let shard_state = &mut ctx.accounts.shard_state;
    
    // Build execution context
    let execution_context = ExecutionContext::new(
        capability_id.clone(),
        ctx.accounts.caller.key(),
        None, // No session for now
    ).with_input_data(input_data.clone());
    
    // Execute using embedded eval logic
    let _result = shard_state.process_execute_capability(
        capability_id,
        input_data,
        &execution_context,
    )?;
    
    msg!("Capability executed successfully");
    Ok(())
}

/// Update evaluation configuration
pub fn update_eval_config(
    ctx: Context<UpdateEvalConfig>,
    new_config: EvalConfig,
) -> Result<()> {
    let shard_state = &mut ctx.accounts.shard_state;
    shard_state.update_eval_config(new_config)?;
    
    msg!("Eval configuration updated");
    Ok(())
}

/// Set ordering rules for the shard
pub fn set_ordering_rules(
    _ctx: Context<SetOrderingRules>,
    partial_order: PartialOrder,
) -> Result<()> {
    // Validate the partial order
    partial_order.validate_consistency()?;
    
    // TODO: Store the partial order in shard state
    // For now, just log the operation
    msg!("Ordering rules set for shard with {} constraints", partial_order.constraints.len());
    Ok(())
}

// ========== ORIGINAL NAMESPACE INSTRUCTIONS ==========

/// Accounts for validating namespace scope
#[derive(Accounts)]
pub struct ValidateNamespaceScope<'info> {
    pub signer: Signer<'info>,
}

/// Accounts for checking object access
#[derive(Accounts)]
pub struct CheckObjectAccess<'info> {
    pub signer: Signer<'info>,
}

/// Accounts for verifying capability composition
#[derive(Accounts)]
pub struct VerifyCapabilityComposition<'info> {
    pub signer: Signer<'info>,
}

/// Validate that a function's namespace capability is valid and diffs respect boundaries
/// This is a pure verification function with no side effects
pub fn validate_namespace_scope(
    _ctx: Context<ValidateNamespaceScope>,
    namespace_capability: NamespaceCapability,
    _function_input: FunctionInput,
    proposed_diffs: Vec<Diff>,
) -> Result<()> {
    // Verify capability integrity
    require!(
        namespace_capability.verify_integrity(),
        NamespaceScopingError::NamespaceInvalidCapability
    );
    
    // Validate diffs against namespace capability
    namespace_capability.validate_diffs(&proposed_diffs)?;
    
    msg!(
        "Namespace scope validation passed: namespace={}, objects={}, diffs={}",
        namespace_capability.namespace_id,
        namespace_capability.accessible_objects.len(),
        proposed_diffs.len()
    );
    
    Ok(())
}

/// Pure function to check if an object is accessible within a namespace
/// No accounts needed - this is a computational verification only
pub fn check_object_access(
    _ctx: Context<CheckObjectAccess>,
    namespace_capability: NamespaceCapability,
    target_object: ObjectId,
) -> Result<bool> {
    // Verify capability integrity
    require!(
        namespace_capability.verify_integrity(),
        NamespaceScopingError::NamespaceInvalidCapability
    );
    
    // Check object access
    let can_access = namespace_capability.can_access_object(&target_object) &&
                    namespace_capability.is_object_type_allowed(&target_object.object_type);
    
    msg!(
        "Object access check: namespace={}, object={:?}, access={}",
        namespace_capability.namespace_id,
        target_object,
        can_access
    );
    
    Ok(can_access)
}

/// Verify that multiple namespace capabilities don't conflict
/// Useful for function composition with multiple capabilities
pub fn verify_capability_composition(
    _ctx: Context<VerifyCapabilityComposition>,
    capabilities: Vec<NamespaceCapability>,
    proposed_diffs: Vec<Diff>,
) -> Result<()> {
    // Verify all capabilities are valid
    for capability in &capabilities {
        require!(
            capability.verify_integrity(),
            NamespaceScopingError::NamespaceInvalidCapability
        );
    }
    
    // Check each diff against at least one capability
    for diff in &proposed_diffs {
        let mut authorized = false;
        
        for capability in &capabilities {
            if capability.can_access_object(&diff.target_object) &&
               capability.is_object_type_allowed(&diff.target_object.object_type) {
                authorized = true;
                break;
            }
        }
        
        require!(
            authorized,
            NamespaceScopingError::NamespaceObjectNotInNamespace
        );
    }
    
    msg!(
        "Capability composition verified: {} capabilities, {} diffs",
        capabilities.len(),
        proposed_diffs.len()
    );
    
    Ok(())
} 