//! Session management logic (controller)

use anchor_lang::prelude::*;
use crate::{Session, SessionConsumption, ShardError, ID, capabilities::*};

/// Create a new session from multiple accounts (new session concept)
pub fn create_session(
    ctx: Context<CreateSession>,
    accounts: Vec<Pubkey>,
    namespace: String,
    nonce: u64,
    metadata: Vec<u8>,
) -> Result<()> {
    // Validate inputs
    require!(
        !accounts.is_empty() && accounts.len() <= 10,
        ShardError::InvalidSessionRequest
    );
    
    require!(
        !namespace.is_empty() && namespace.len() <= 64,
        ShardError::InvalidSessionRequest
    );
    
    // Validate that all accounts exist and are owned by the session creator
    // Also aggregate capabilities and state from all accounts
    let mut aggregated_capabilities = Capabilities::none();
    let mut aggregated_state = [0u8; 32];
    
    for account_pubkey in accounts.iter() {
        // Find the account in remaining_accounts
        let mut found = false;
        for account_info in ctx.remaining_accounts.iter() {
            if account_info.key == account_pubkey {
                // Verify it's a ValenceAccount
                if account_info.owner != &ID {
                    return Err(ShardError::InvalidSessionRequest.into());
                }
                
                // Deserialize and verify ownership
                let account_data = account_info.try_borrow_data()?;
                if account_data.len() < 8 + 32 + 32 { // discriminator + id + owner minimum
                    return Err(ShardError::InvalidSessionRequest.into());
                }
                
                // Check owner field (at offset 8 + 32)
                let owner_bytes = &account_data[40..72];
                let account_owner = Pubkey::new_from_array(owner_bytes.try_into().unwrap());
                
                require!(
                    account_owner == ctx.accounts.owner.key(),
                    ShardError::Unauthorized
                );
                
                // Extract capabilities from account (at offset 8 + 32 + 32 = 72)
                if account_data.len() >= 72 + 4 { // Has capabilities vector
                    let capabilities_offset = 72;
                    let len_bytes = &account_data[capabilities_offset..capabilities_offset + 4];
                    let vec_len = u32::from_le_bytes(len_bytes.try_into().unwrap()) as usize;
                    
                    let mut account_capabilities = Vec::new();
                    let mut offset = capabilities_offset + 4;
                    
                    for _ in 0..vec_len {
                        if offset + 4 > account_data.len() { break; }
                        
                        let str_len_bytes = &account_data[offset..offset + 4];
                        let str_len = u32::from_le_bytes(str_len_bytes.try_into().unwrap()) as usize;
                        offset += 4;
                        
                        if offset + str_len > account_data.len() { break; }
                        
                        let cap_bytes = &account_data[offset..offset + str_len];
                        if let Ok(capability) = String::from_utf8(cap_bytes.to_vec()) {
                            account_capabilities.push(capability);
                        }
                        offset += str_len;
                    }
                    
                    // Convert string capabilities to bitmap and merge
                    let account_caps = Capabilities::from_strings(&account_capabilities);
                    aggregated_capabilities.merge(account_caps);
                }
                
                // Extract state hash from account
                // State hash is at offset 8 + 32 + 32 + 200 = 272
                if account_data.len() >= 272 + 32 {
                    let state_hash_offset = 272;
                    let state_hash_bytes = &account_data[state_hash_offset..state_hash_offset + 32];
                    let account_state_hash: [u8; 32] = state_hash_bytes.try_into().unwrap();
                    
                    // Aggregate using XOR
                    for i in 0..32 {
                        aggregated_state[i] ^= account_state_hash[i];
                    }
                }
                
                found = true;
                break;
            }
        }
        
        require!(found, ShardError::InvalidSessionRequest);
    }
    
    // Create session
    let session_key = ctx.accounts.session.key();
    let session = &mut ctx.accounts.session;
    session.id = session_key;
    session.owner = ctx.accounts.owner.key();
    session._internal_accounts = accounts;
    session.namespace = namespace;
    session.is_consumed = false;
    session.nonce = nonce;
    session.created_at = Clock::get()?.unix_timestamp;
    session.metadata = metadata;
    session.capabilities = aggregated_capabilities.0; // Store the aggregated capabilities bitmap
    session.state_root = aggregated_state; // Store the aggregated state root
    
    msg!("Session created with {} accounts in namespace '{}', capabilities: {:064b}, state_root: {:?}", 
         session._internal_accounts.len(), session.namespace, session.capabilities, session.state_root);
    Ok(())
}

/// Consume a session and create new sessions (UTXO-like semantics)
pub fn consume_session(
    ctx: Context<ConsumeSession>,
    new_sessions_data: Vec<(Vec<Pubkey>, String, u64, Vec<u8>)>, // (accounts, namespace, nonce, metadata)
) -> Result<()> {
    let old_session = &mut ctx.accounts.old_session;
    
    // Verify session is not already consumed
    require!(
        !old_session.is_consumed,
        ShardError::SessionAlreadyConsumed
    );
    
    // Mark session as consumed
    old_session.is_consumed = true;
    
    // Record the consumption
    let consumption = &mut ctx.accounts.consumption_record;
    consumption.consumed_session = old_session.id;
    consumption.created_sessions = Vec::new(); // Will be filled by subsequent create_session calls
    
    // Get transaction signature from Sysvar
    let current_ix = anchor_lang::solana_program::sysvar::instructions::get_instruction_relative(0, &ctx.accounts.instructions_sysvar)?;
    let mut tx_signature = [0u8; 64];
    // Use the first 64 bytes of instruction data as a pseudo-signature
    // In production, this would be the actual transaction signature
    if current_ix.data.len() >= 64 {
        tx_signature.copy_from_slice(&current_ix.data[..64]);
    } else {
        // Use a hash of available data as signature
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hasher::write(&mut hasher, &current_ix.data);
        std::hash::Hasher::write(&mut hasher, &old_session.id.to_bytes());
        let hash = std::hash::Hasher::finish(&hasher);
        tx_signature[..8].copy_from_slice(&hash.to_le_bytes());
    }
    
    consumption.transaction_signature = tx_signature;
    consumption.consumed_at = Clock::get()?.unix_timestamp;
    
    msg!("Session {} consumed, creating {} new sessions", 
         old_session.id, new_sessions_data.len());
    
    // Note: New sessions would be created in separate instructions
    // This provides the consumption atomicity needed for UTXO semantics
    Ok(())
}

// Account contexts

#[derive(Accounts)]
pub struct CreateSession<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 32 + 4 + (10 * 32) + 4 + 64 + 1 + 8 + 8 + 4 + 256 + 8 + 32, // discriminator + id + owner + accounts_vec + namespace + consumed + nonce + created_at + metadata_vec + capabilities + state_root
    )]
    pub session: Account<'info, Session>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ConsumeSession<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        mut,
        constraint = old_session.owner == owner.key() @ ShardError::Unauthorized,
        constraint = !old_session.is_consumed @ ShardError::SessionAlreadyConsumed,
    )]
    pub old_session: Account<'info, Session>,
    
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 4 + (10 * 32) + 64 + 8, // discriminator + consumed_session + created_sessions_vec + tx_signature + consumed_at
    )]
    pub consumption_record: Account<'info, SessionConsumption>,
    
    /// Instructions sysvar for getting transaction info
    /// CHECK: This is the instructions sysvar
    pub instructions_sysvar: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
} 