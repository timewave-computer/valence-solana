use anchor_lang::prelude::*;
use crate::error::RegistryError;

/// Registry Program state
#[account]
pub struct RegistryState {
    /// Program owner
    pub owner: Pubkey,
    /// Authorization Program ID
    pub authorization_program_id: Pubkey,
    /// Account Factory address
    pub account_factory: Pubkey,
    /// Bump seed for PDA
    pub bump: u8,
}

impl<'info> Initialize
UpdateLibraryStatus
QueryLibrary
ListLibraries<'info> {
    pub fn try_accounts(
        ctx: &Context<'_, '_, '_, 'info, Initialize
UpdateLibraryStatus
QueryLibrary
ListLibraries<'info>>,
        _bumps: &anchor_lang::prelude::BTreeMap<String, u8>,
    ) -> Result<()> {
        // Additional validation logic can be added here if needed
        Ok(())
    }
}


/// Library information
#[account]
pub struct LibraryInfo {
    /// Program ID of the library
    pub program_id: Pubkey,
    /// Type of library (e.g., "token_transfer", "vault_deposit")
    pub library_type: String,
    /// Human-readable description
    pub description: String,
    /// Whether the library is globally approved
    pub is_approved: bool,
    /// Version information
    pub version: String,
    /// Last updated timestamp
    pub last_updated: i64,
    /// Bump seed for PDA
    pub bump: u8,
}

/// Instruction context for initializing the registry program
#[derive(Accounts)]
pub struct Initialize<'info> {
    /// The program state account
    #[account(
        init,
        payer = owner,
        space = 8 + std::mem::size_of::<RegistryState>(),
        seeds = [b"registry_state".as_ref()],
        bump
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    /// The account paying for the initialization
    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// System program for creating accounts
    pub system_program: Program<'info, System>,
}

/// Instruction context for registering a library
#[derive(Accounts)]
#[instruction(library_type: String, description: String)]
pub struct RegisterLibrary<'info> {
    /// The program state account
    #[account(
        seeds = [b"registry_state".as_ref()],
        bump = registry_state.bump,
        constraint = registry_state.owner == owner.key() @ RegistryError::NotAuthorized
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    /// The library info account
    #[account(
        init,
        payer = owner,
        space = 8 + std::mem::size_of::<LibraryInfo>() + 
                library_type.len() + 
                description.len() + 
                32, // Extra space for version
        seeds = [b"library_info".as_ref(), program_id.key().as_ref()],
        bump
    )]
    pub library_info: Account<'info, LibraryInfo>,
    
    /// The program ID of the library being registered
    /// This is not a signer, just the public key
    pub program_id: UncheckedAccount<'info>,
    
    /// The owner of the registry
    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// System program for creating accounts
    pub system_program: Program<'info, System>,
}

/// Instruction context for updating a library's status
#[derive(Accounts)]
pub struct UpdateLibraryStatus<'info> {
    /// The program state account
    #[account(
        seeds = [b"registry_state".as_ref()],
        bump = registry_state.bump,
        constraint = registry_state.owner == owner.key() @ RegistryError::NotAuthorized
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    /// The library info account
    #[account(
        mut,
        seeds = [b"library_info".as_ref(), library_info.program_id.as_ref()],
        bump = library_info.bump
    )]
    pub library_info: Account<'info, LibraryInfo>,
    
    /// The owner of the registry
    pub owner: Signer<'info>,
}

/// Instruction context for querying a library
#[derive(Accounts)]
pub struct QueryLibrary<'info> {
    /// The program state account
    #[account(
        seeds = [b"registry_state".as_ref()],
        bump = registry_state.bump
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    /// The library info account
    #[account(
        seeds = [b"library_info".as_ref(), program_id.key().as_ref()],
        bump = library_info.bump
    )]
    pub library_info: Account<'info, LibraryInfo>,
    
    /// The program ID to query
    pub program_id: UncheckedAccount<'info>,
}

/// Instruction context for listing libraries
#[derive(Accounts)]
pub struct ListLibraries<'info> {
    /// The program state account
    #[account(
        seeds = [b"registry_state".as_ref()],
        bump = registry_state.bump
    )]
    pub registry_state: Account<'info, RegistryState>,
} 