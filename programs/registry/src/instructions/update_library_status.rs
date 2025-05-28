use anchor_lang::prelude::*;
use crate::state::*;
use crate::cache::LibraryCache;
use std::cell::RefCell;

// Thread-local storage for the library cache
thread_local! {
    static LIBRARY_CACHE: RefCell<LibraryCache> = RefCell::new(LibraryCache::new());
}

pub fn handler(
    ctx: Context<UpdateLibraryStatus>,
    is_approved: bool,
) -> Result<()> {
    // Get the library info account
    let library_info = &mut ctx.accounts.library_info;
    
    // Update the approval status
    library_info.is_approved = is_approved;
    
    // Update the last updated timestamp
    library_info.last_updated = Clock::get()?.unix_timestamp;
    
    // Update cache with the current library for future lookups
    LIBRARY_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.update_approval_status(&library_info.program_id, is_approved);
    });
    
    // Log the status update
    msg!(
        "Library status updated: Program ID: {}, Type: {}, Approved: {}",
        library_info.program_id,
        library_info.library_type,
        library_info.is_approved
    );
    
    Ok(())
} 