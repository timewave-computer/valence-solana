// Namespace management instructions for valence-kernel hierarchical organization
//
// This module implements namespace creation, management, and state storage operations
// that enable hierarchical organization of sessions and protocols within the kernel.
// Namespaces provide the foundation for multi-tenant operation and security isolation
// across different protocol deployments and organizational boundaries.
//
// KERNEL INTEGRATION: The namespace system integrates with session creation and
// management, providing the hierarchical context for PDA derivation, permission
// inheritance, and cross-namespace access control during batch execution.
//
// SECURITY BOUNDARIES: Namespace operations enforce creation privileges and
// implement the one-way trust model where parent namespaces can access child
// state but children cannot access parent state, ensuring security isolation.

use anchor_lang::prelude::*;
use crate::namespace::{Namespace, NamespacePath};
use crate::errors::KernelError;

// ================================
// Create Namespace Instruction
// ================================

#[derive(Accounts)]
#[instruction(namespace_path: NamespacePath, max_state_size: u32)]
pub struct CreateNamespace<'info> {
    #[account(
        init,
        payer = authority,
        space = Namespace::space(),
        seeds = [Namespace::SEED_PREFIX, namespace_path.path[..namespace_path.len as usize].as_ref()],
        bump
    )]
    pub namespace: Box<Account<'info, Namespace>>,

    /// Parent namespace (optional, for 2-level hierarchy)
    pub parent: Option<Box<Account<'info, Namespace>>>,

    /// Authority creating the namespace
    #[account(mut)]
    pub authority: Signer<'info>,

    /// System program
    pub system_program: Program<'info, System>,
}

/// Create a new namespace in the hierarchy
/// 
/// # Errors
/// Returns errors for invalid namespace paths or unauthorized creation
#[allow(clippy::needless_pass_by_value)]
pub fn create_namespace(
    ctx: Context<CreateNamespace>,
    namespace_path: NamespacePath,
    max_state_size: u32,
) -> Result<()> {
    let namespace = &mut ctx.accounts.namespace;
    let clock = Clock::get()?;
    
    // Enforce 2-level hierarchy maximum
    require!(
        namespace_path.depth() <= 2,
        KernelError::NamespaceInvalidPath
    );
    
    // If this has a parent, validate it
    if namespace_path.depth() == 2 {
        let parent = ctx.accounts.parent
            .as_ref()
            .ok_or(KernelError::NamespaceInvalidPath)?;
            
        // Verify parent path matches
        let expected_parent = namespace_path.parent()
            .ok_or(KernelError::NamespaceInvalidPath)?;
        
        require!(
            parent.path == expected_parent,
            KernelError::NamespaceInvalidPath
        );
        
        // Only parent owner can create children
        require!(
            parent.owner == ctx.accounts.authority.key(),
            KernelError::Unauthorized
        );
    }
    
    // Initialize namespace
    let new_namespace = Namespace::new(
        namespace_path,
        ctx.accounts.parent.as_ref().map(|p| p.key()),
        ctx.accounts.authority.key(),
        max_state_size,
        &clock,
    )?;
    namespace.set_inner(new_namespace);
    
    Ok(())
}

// ================================
// Delete Namespace Instruction
// ================================

#[derive(Accounts)]
pub struct DeleteNamespace<'info> {
    #[account(
        mut,
        close = authority,
        constraint = namespace.child_count == 0 @ KernelError::NamespaceHasChildren
    )]
    pub namespace: Box<Account<'info, Namespace>>,

    /// Authority must be owner
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// Delete an empty namespace
/// 
/// # Errors
/// Returns errors for unauthorized deletion or non-empty namespaces
#[allow(clippy::needless_pass_by_value)]
pub fn delete_namespace(ctx: Context<DeleteNamespace>) -> Result<()> {
    require!(
        ctx.accounts.namespace.owner == ctx.accounts.authority.key(),
        KernelError::Unauthorized
    );
    
    // Account will be closed automatically by anchor
    Ok(())
}

// ================================
// Read Namespace State
// ================================

#[derive(Accounts)]
pub struct ReadNamespaceState<'info> {
    pub namespace: Box<Account<'info, Namespace>>,
}

/// Read namespace state data
/// 
/// # Errors
/// Returns errors for invalid namespace access
#[allow(clippy::needless_pass_by_value)]
pub fn read_namespace_state(ctx: Context<ReadNamespaceState>) -> Result<Vec<u8>> {
    Ok(ctx.accounts.namespace.read_state().to_vec())
}

// ================================
// Write Namespace State
// ================================

#[derive(Accounts)]
pub struct WriteNamespaceState<'info> {
    #[account(mut)]
    pub namespace: Box<Account<'info, Namespace>>,

    /// Authority must be owner
    pub authority: Signer<'info>,
}

/// Write namespace state data
/// 
/// # Errors
/// Returns errors for unauthorized writes or invalid data
#[allow(clippy::needless_pass_by_value)]
pub fn write_namespace_state(
    ctx: Context<WriteNamespaceState>,
    data: &[u8],
) -> Result<()> {
    let namespace = &mut ctx.accounts.namespace;
    
    // Only owner can write
    require!(
        namespace.owner == ctx.accounts.authority.key(),
        KernelError::Unauthorized
    );
    
    namespace.write_state(data)?;
    Ok(())
}

// ================================
// Update Namespace State
// ================================

/// Update operation for partial state modifications
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct StateUpdate {
    /// Offset in state array to start writing
    pub offset: u32,
    /// Fixed-size data buffer
    pub data: [u8; 64],
    /// Actual data length
    pub data_len: u8,
}

#[derive(Accounts)]
pub struct UpdateNamespaceState<'info> {
    #[account(mut)]
    pub namespace: Box<Account<'info, Namespace>>,

    /// Authority must be owner
    pub authority: Signer<'info>,
}

/// Update namespace state with partial modifications
/// 
/// # Errors
/// Returns errors for unauthorized updates or invalid offsets
#[allow(clippy::needless_pass_by_value)]
pub fn update_namespace_state(
    ctx: Context<UpdateNamespaceState>,
    updates: &[StateUpdate],
) -> Result<()> {
    let namespace = &mut ctx.accounts.namespace;
    
    // Only owner can update
    require!(
        namespace.owner == ctx.accounts.authority.key(),
        KernelError::Unauthorized
    );
    
    // Apply updates
    for update in updates {
        namespace.update_state(
            update.offset, 
            &update.data[..update.data_len as usize]
        )?;
    }
    
    Ok(())
}