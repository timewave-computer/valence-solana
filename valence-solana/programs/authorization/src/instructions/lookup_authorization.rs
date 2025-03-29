use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::AuthorizationError;
use crate::cache::{AuthorizationCache, helpers};

// Use the shared thread-local cache
thread_local! {
    static AUTHORIZATION_CACHE: RefCell<AuthorizationCache> = RefCell::new(AuthorizationCache::new());
}

/// Instruction context for looking up an authorization
#[derive(Accounts)]
#[instruction(label: String)]
pub struct LookupAuthorization<'info> {
    /// The program state account
    #[account(
        seeds = [b"authorization_state".as_ref()],
        bump,
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
    
    /// The authorization to look up (optional - if not found by cache)
    /// We use Option to make it optional in the CPI call
    pub authorization: Option<Account<'info, Authorization>>,
}

pub fn handler(ctx: Context<LookupAuthorization>, label: String) -> Result<Pubkey> {
    let program_id = ctx.program_id;
    
    // Try to find in cache first
    let mut found_address = None;
    
    AUTHORIZATION_CACHE.with(|cache| {
        let mut cache_ref = cache.borrow_mut();
        if let Some(address) = cache_ref.get_address(&label) {
            found_address = Some(address);
            return;
        }
        
        // If authorization is provided, add it to the cache
        if let Some(auth) = &ctx.accounts.authorization {
            if auth.label == label {
                cache_ref.add_authorization(auth);
                found_address = Some(auth.key());
                return;
            }
        }
        
        // Try to compute the PDA
        let (address, bump) = Pubkey::find_program_address(
            &[b"authorization".as_ref(), label.as_bytes()],
            program_id,
        );
        
        // Add to cache for future lookups
        cache_ref.label_to_address.insert(label.clone(), address);
        cache_ref.pda_bumps.insert(label.clone(), bump);
        
        found_address = Some(address);
    });
    
    // We should always have an address at this point
    if let Some(address) = found_address {
        msg!("Found authorization address: {}", address);
        Ok(address)
    } else {
        Err(error!(AuthorizationError::AuthorizationNotFound))
    }
} 