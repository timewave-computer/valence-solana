use anchor_lang::prelude::*;

// Program ID declaration
declare_id!("RegistryProgramxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");

pub mod state;
pub mod instructions;
pub mod error;
pub mod cache;

use state::*;
use instructions::*;
use error::*;

#[program]
pub mod registry {
    use super::*;

    /// Initialize the Registry Program with an owner and authorization program
    pub fn initialize(
        ctx: Context<Initialize>,
        authorization_program_id: Pubkey,
        account_factory: Pubkey,
    ) -> Result<()> {
        instructions::initialize::handler(ctx, authorization_program_id, account_factory)
    }

    /// Register a new library with the registry
    pub fn register_library(
        ctx: Context<RegisterLibrary>,
        library_type: String,
        description: String,
        is_approved: bool,
    ) -> Result<()> {
        instructions::register_library::handler(ctx, library_type, description, is_approved)
    }

    /// Update an existing library's status
    pub fn update_library_status(
        ctx: Context<UpdateLibraryStatus>,
        is_approved: bool,
    ) -> Result<()> {
        instructions::update_library_status::handler(ctx, is_approved)
    }

    /// Query a library's information
    pub fn query_library(
        ctx: Context<QueryLibrary>,
    ) -> Result<LibraryInfo> {
        instructions::query_library::handler(ctx)
    }

    /// List approved libraries with pagination
    pub fn list_libraries(
        ctx: Context<ListLibraries>,
        start_after: Option<Pubkey>,
        limit: u8,
    ) -> Result<Vec<LibraryInfo>> {
        instructions::list_libraries::handler(ctx, start_after, limit)
    }
}

/// The source file structure will be:
/// 
/// lib.rs - Main program entry point with instruction routing
/// state.rs - Account structures and data types
/// error.rs - Error handling for the program
/// instructions/ - Individual instruction handlers
///    mod.rs - Module exports
///    initialize.rs - Handler for initialize instruction
///    register_library.rs - Handler for library registration
///    update_library_status.rs - Handler for updating library status
///    query_library.rs - Handler for querying library information
/// 