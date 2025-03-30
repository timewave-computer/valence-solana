use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::RegistryError;
use crate::cache::{LibraryCache, helpers};
use std::cell::RefCell;

// Thread-local storage for the library cache
thread_local! {
    static LIBRARY_CACHE: RefCell<LibraryCache> = RefCell::new(LibraryCache::new());
}

pub fn handler(
    ctx: Context<QueryLibrary>,
) -> Result<LibraryInfo> {
    // Get the library info account
    let library_info = &ctx.accounts.library_info;
    
    // Check that the program ID matches
    if library_info.program_id != ctx.accounts.program_id.key() {
        return Err(error!(RegistryError::LibraryNotFound));
    }
    
    // Update cache with the current library for future lookups
    LIBRARY_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.add_library(library_info);
    });
    
    // Log the query
    msg!(
        "Library queried: Program ID: {}, Type: {}, Approved: {}",
        library_info.program_id,
        library_info.library_type,
        library_info.is_approved
    );
    
    // Return a copy of the library info
    Ok(LibraryInfo {
        program_id: library_info.program_id,
        library_type: library_info.library_type.clone(),
        description: library_info.description.clone(),
        is_approved: library_info.is_approved,
        version: library_info.version.clone(),
        last_updated: library_info.last_updated,
        bump: library_info.bump,
    })
} 