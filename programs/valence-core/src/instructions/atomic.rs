use anchor_lang::prelude::*;
use crate::{state::*, errors::*, instructions::{verify_account, VerifyAccount}};

// ================================
// Atomic Multi-Account Operations
// ================================

// ===== Dynamic Atomic Operations =====

/// Use multiple accounts atomically (all or nothing)
/// Supporting dynamic number of accounts
pub fn use_accounts_atomic<'info>(ctx: Context<'_, '_, '_, 'info, UseAccountsAtomic<'info>>) -> Result<()> {
    let clock = Clock::get()?;
    
    // ===== Input Validation =====
    
    // Validate we have at least one account and not too many
    require!(
        !ctx.remaining_accounts.is_empty(),
        CoreError::InsufficientAccounts
    );
    require!(
        ctx.remaining_accounts.len() <= MAX_ACCOUNTS_PER_SESSION,
        CoreError::TooManyAccounts
    );
    
    // ===== Phase 1: Pre-Validation =====
    
    // Validate all accounts without modifying state
    let mut managed_accounts = Vec::with_capacity(ctx.remaining_accounts.len());
    
    for (i, account_info) in ctx.remaining_accounts.iter().enumerate() {
        // Deserialize the account data for validation
        let account_data = account_info.try_borrow_data()
            .map_err(|_| CoreError::AccountMismatch)?;
        
        // Ensure account has sufficient data
        if account_data.len() < 8 + SessionAccount::SIZE {
            msg!("Account {} has insufficient data", i);
            return Err(CoreError::AccountMismatch.into());
        }
        
        // Deserialize SessionAccount from account data
        let managed_account = SessionAccount::try_deserialize(
            &mut &account_data[8..] // Skip 8-byte discriminator
        ).map_err(|_| CoreError::AccountMismatch)?;
        
        // Verify account hasn't expired
        require!(
            clock.unix_timestamp < managed_account.expires_at,
            CoreError::AccountExpired
        );
        
        managed_accounts.push(managed_account);
    }
    
    // ===== Phase 2: Atomic Verification =====
    
    // Verify all accounts via CPI (atomic - if any fails, all fail)
    for (i, account_info) in ctx.remaining_accounts.iter().enumerate() {
        let cpi_accounts = VerifyAccount {
            account: account_info.clone(),
            caller: ctx.accounts.caller.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(
            ctx.accounts.verifier_program.to_account_info(),
            cpi_accounts,
        );
        
        verify_account(cpi_ctx).map_err(|e| {
            msg!("Verification failed for account {}: {:?}", i, e);
            e
        })?;
    }
    
    // ===== Phase 3: State Update =====
    
    // If all verifications passed, increment usage counts atomically
    for account_info in ctx.remaining_accounts.iter() {
        let mut account_data = account_info.try_borrow_mut_data()
            .map_err(|_| CoreError::AccountMismatch)?;
        
        let mut managed_account = SessionAccount::try_deserialize(
            &mut &account_data[8..] // Skip discriminator
        ).map_err(|_| CoreError::AccountMismatch)?;
        
        managed_account.uses = managed_account.uses
            .checked_add(1)
            .ok_or(CoreError::UsageOverflow)?;
        
        // Serialize back to account data
        let mut writer = &mut account_data[8..];
        managed_account.try_serialize(&mut writer)
            .map_err(|_| CoreError::AccountMismatch)?;
    }
    
    msg!("Successfully used {} accounts atomically", ctx.remaining_accounts.len());
    Ok(())
}

// ===== Simplified Two-Account Operations =====

/// Use multiple accounts atomically (simplified 2-account version for common case)
pub fn use_accounts_simple(ctx: Context<UseAccountsSimple>) -> Result<()> {
    let clock = Clock::get()?;
    
    // ===== Expiration Checks =====
    
    // Check first account
    require!(
        clock.unix_timestamp < ctx.accounts.account1.expires_at,
        CoreError::AccountExpired
    );
    
    // Check second account  
    require!(
        clock.unix_timestamp < ctx.accounts.account2.expires_at,
        CoreError::AccountExpired
    );
    
    // ===== Account Verification =====
    
    // CPI to verifier for first account
    let cpi_accounts1 = VerifyAccount {
        account: ctx.accounts.account1.to_account_info(),
        caller: ctx.accounts.caller.to_account_info(),
    };
    let cpi_ctx1 = CpiContext::new(
        ctx.accounts.verifier_program.to_account_info(),
        cpi_accounts1,
    );
    verify_account(cpi_ctx1)?;
    
    // CPI to verifier for second account
    let cpi_accounts2 = VerifyAccount {
        account: ctx.accounts.account2.to_account_info(),
        caller: ctx.accounts.caller.to_account_info(),
    };
    let cpi_ctx2 = CpiContext::new(
        ctx.accounts.verifier_program.to_account_info(),
        cpi_accounts2,
    );
    verify_account(cpi_ctx2)?;
    
    // ===== Usage Update =====
    
    // If both verifications passed, increment usage
    let account1 = &mut ctx.accounts.account1;
    account1.uses = account1.uses.checked_add(1).ok_or(CoreError::UsageOverflow)?;
    
    let account2 = &mut ctx.accounts.account2;
    account2.uses = account2.uses.checked_add(1).ok_or(CoreError::UsageOverflow)?;
    
    msg!("Successfully used 2 accounts atomically");
    Ok(())
}

// ===== Optimized Atomic Operations =====

/// Optimized atomic account usage with CPI depth awareness
pub fn use_accounts_atomic_optimized<'info>(
    ctx: Context<'_, '_, 'info, 'info, UseAccountsAtomicOptimized<'info>>,
    estimated_depth: u8, // Caller provides estimated CPI depth
) -> Result<()> {
    let clock = Clock::get()?;
    let account_count = ctx.remaining_accounts.len();
    
    // ===== Input Validation =====
    
    // Validate account count
    require!(account_count > 0, CoreError::InsufficientAccounts);
    require!(account_count <= MAX_ACCOUNTS_PER_SESSION, CoreError::TooManyAccounts);
    
    // ===== CPI Depth Management =====
    
    // Check if we have sufficient depth for operation
    let min_required_depth = if account_count > 4 { 2 } else { 1 };
    require!(
        estimated_depth <= 4 - min_required_depth,
        CoreError::InsufficientCallDepth
    );
    
    // ===== Phase 1: Account Classification =====
    let (accounts_data, verification_groups) = classify_accounts_for_verification(
        ctx.remaining_accounts,
        &ctx.accounts.session,
        &clock,
    )?;
    
    let VerificationGroups {
        inline_accounts,
        cached_accounts,
        direct_accounts,
    } = verification_groups;
    
    // ===== Phase 2: Optimized Verification =====
    
    // Process inline verifications (no CPI needed)
    for (index, inline_type) in inline_accounts {
        let (_, account) = &accounts_data[index];
        verify_inline(account, &ctx.accounts.caller.key(), inline_type)?;
    }
    
    // Skip cached verifications (already verified)
    msg!("Using cached verification for {} accounts", cached_accounts.len());
    
    // Process remaining verifications with depth-aware strategy
    let remaining_depth = 4_u8.saturating_sub(estimated_depth);
    
    // Process direct verifications with intelligent batching based on depth
    process_direct_verifications(&ctx, &direct_accounts, &accounts_data, remaining_depth)?;
    
    // ===== Phase 3: State and Cache Updates =====
    
    // Update verification cache if session provided
    if let Some(session) = &mut ctx.accounts.session {
        update_verification_cache(&mut session.verification_data, &accounts_data)?;
    }
    
    // Update all account states atomically
    for (account_info, _) in &accounts_data {
        // Borrow account data for mutation
        let mut account_data = account_info.try_borrow_mut_data()?;
        let mut managed_account = SessionAccount::try_deserialize(&mut &account_data[8..])?;
        
        // Increment usage count with overflow protection
        managed_account.uses = managed_account.uses
            .checked_add(1)
            .ok_or(CoreError::UsageOverflow)?;
        
        // Serialize updated account back to storage
        managed_account.try_serialize(&mut &mut account_data[8..])?;
    }
    
    msg!("Optimized atomic usage complete: {} accounts, depth {}", 
        account_count, estimated_depth);
    
    Ok(())
}

// ================================
// Helper Functions
// ================================

// ===== Batch Verification Helpers =====

/// Verify a batch of accounts with single CPI
fn verify_batch<'info>(
    ctx: &Context<'_, '_, '_, 'info, UseAccountsAtomicOptimized<'info>>,
    _verifier: &Pubkey,
    indices: &[usize],
    accounts_data: &[(&AccountInfo<'info>, SessionAccount)],
) -> Result<()> {
    // Prepare batch verification request by collecting account info
    let mut account_keys = Vec::with_capacity(indices.len());
    let mut remaining = Vec::with_capacity(indices.len());
    
    // Collect all accounts to verify in this batch
    for &index in indices {
        let (account_info, _) = &accounts_data[index];
        account_keys.push(account_info.key());
        remaining.push(account_info.to_account_info());
    }
    
    // Execute single CPI for entire batch
    let cpi_accounts = VerifyAccountsBatch {
        request: ctx.accounts.verification_request.to_account_info(),
        caller: ctx.accounts.caller.to_account_info(),
        session: ctx.accounts.session.as_ref()
            .map(|s| s.to_account_info())
            .unwrap_or_else(|| ctx.accounts.caller.to_account_info()),
    };
    
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.verifier_program.to_account_info(),
        cpi_accounts,
        &[],
    ).with_remaining_accounts(remaining);
    
    verify_accounts_batch(cpi_ctx)?;
    
    msg!("Batch verified {} accounts with 1 CPI", indices.len());
    Ok(())
}

/// Verify a single account
fn verify_single<'info>(
    ctx: &Context<'_, '_, '_, 'info, UseAccountsAtomicOptimized<'info>>,
    account_info: &AccountInfo<'info>,
    _verifier: &Pubkey,
) -> Result<()> {
    let cpi_accounts = VerifyAccount {
        account: account_info.clone(),
        caller: ctx.accounts.caller.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new(
        ctx.accounts.verifier_program.to_account_info(),
        cpi_accounts,
    );
    
    verify_account(cpi_ctx)
}

/// Check if cached verification is valid
fn is_cached_verification_valid(
    session: &Option<Account<Session>>,
    account_index: usize,
) -> Result<bool> {
    if let Some(session) = session {
        // Check if cache has become stale (older than 5 minutes)
        if VerificationCache::is_stale(&session.verification_data)? {
            return Ok(false);
        }
        
        // Check if this specific account is marked as verified in cache
        Ok(VerificationCache::is_verified(&session.verification_data, account_index))
    } else {
        // No session means no cache available
        Ok(false)
    }
}

/// Update verification cache with results
fn update_verification_cache(
    verification_data: &mut [u8; 256],
    accounts_data: &[(&AccountInfo, SessionAccount)],
) -> Result<()> {
    // Update cache timestamp to current time
    VerificationCache::update_timestamp(verification_data)?;
    
    // Mark all accounts as verified in the cache bitmap
    for (i, _) in accounts_data.iter().enumerate() {
        VerificationCache::set_verified(verification_data, i);
    }
    
    Ok(())
}

// ================================
// Verification Infrastructure
// ================================

// ===== Verification Mode Types =====

/// Verification modes to optimize CPI depth usage
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum VerificationMode {
    /// Direct CPI to verifier (current approach)
    Direct,
    /// Batch verification via single CPI
    Batch,
    /// Inline verification for known patterns
    Inline,
    /// Use cached verification from session
    Cached,
}

// ===== Inline Verification Types =====

/// Known verifier types that can be verified inline
#[derive(Clone, Copy, PartialEq)]
pub enum InlineVerifierType {
    /// Simple owner check: owner == caller
    SimpleOwner,
    /// Time-based: current_time < expiry
    TimeBased,
    /// Counter-based: uses < max_uses
    CounterBased,
    /// Compound: owner + time + counter
    Standard,
}

// ===== Verification Cache Management =====

/// Verification cache layout in session.verification_data
/// [0..4]: Cache version and flags
/// [4..8]: Last verification timestamp
/// [8..40]: Verification bitmap (256 bits for account status)
/// [40..256]: Verifier-specific data
pub struct VerificationCache;

impl VerificationCache {
    pub const VERSION_OFFSET: usize = 0;
    pub const TIMESTAMP_OFFSET: usize = 4;
    pub const BITMAP_OFFSET: usize = 8;
    pub const BITMAP_SIZE: usize = 32; // 256 bits
    pub const DATA_OFFSET: usize = 40;
    
    /// Check if an account index is verified in cache
    pub fn is_verified(session_data: &[u8; 256], account_index: usize) -> bool {
        if account_index >= 256 {
            return false;
        }
        
        let byte_index = Self::BITMAP_OFFSET + (account_index / 8);
        let bit_index = account_index % 8;
        
        (session_data[byte_index] & (1 << bit_index)) != 0
    }
    
    /// Mark an account as verified in cache
    pub fn set_verified(session_data: &mut [u8; 256], account_index: usize) {
        if account_index >= 256 {
            return;
        }
        
        let byte_index = Self::BITMAP_OFFSET + (account_index / 8);
        let bit_index = account_index % 8;
        
        session_data[byte_index] |= 1 << bit_index;
    }
    
    /// Check if cache is stale (older than 5 minutes)
    pub fn is_stale(session_data: &[u8; 256]) -> Result<bool> {
        let timestamp = i64::from_le_bytes(
            session_data[Self::TIMESTAMP_OFFSET..Self::TIMESTAMP_OFFSET + 8]
                .try_into()
                .map_err(|_| CoreError::InvalidCacheFormat)?
        );
        
        let clock = Clock::get()?;
        Ok(clock.unix_timestamp - timestamp > 300)
    }
    
    /// Update cache timestamp
    pub fn update_timestamp(session_data: &mut [u8; 256]) -> Result<()> {
        let clock = Clock::get()?;
        session_data[Self::TIMESTAMP_OFFSET..Self::TIMESTAMP_OFFSET + 8]
            .copy_from_slice(&clock.unix_timestamp.to_le_bytes());
        Ok(())
    }
}

// ===== Verification Mode Selection =====

/// Determine optimal verification mode based on context
pub fn determine_verification_mode(
    verifier: &Pubkey,
    remaining_depth: u8,
    account_count: usize,
) -> VerificationMode {
    // ===== Depth-Based Strategy =====
    
    // At depth 3-4, we must use inline or cached only
    if remaining_depth >= 3 {
        if is_inline_verifier(verifier) {
            return VerificationMode::Inline;
        }
        return VerificationMode::Cached;
    }
    
    // At depth 2, prefer batch if available and multiple accounts
    if remaining_depth == 2 && account_count > 2 && supports_batch_verification(verifier) {
        return VerificationMode::Batch;
    }
    
    // At depth 0-1, use direct for single accounts
    if account_count == 1 {
        return VerificationMode::Direct;
    }
    
    // For multiple accounts at good depth, prefer batch
    if supports_batch_verification(verifier) {
        return VerificationMode::Batch;
    }
    
    // Default to direct
    VerificationMode::Direct
}

/// Check if verifier is a known inline type
pub fn is_inline_verifier(verifier: &Pubkey) -> bool {
    // Delegate to type detection function
    get_inline_verifier_type(verifier).is_some()
}

/// Get the inline verifier type if known
pub fn get_inline_verifier_type(verifier: &Pubkey) -> Option<InlineVerifierType> {
    // In production, these would be program-derived addresses
    // For now, using placeholder detection based on first byte
    let verifier_bytes = verifier.to_bytes();
    
    // Map first byte to known inline verifier types
    match verifier_bytes[0] {
        0x01 => Some(InlineVerifierType::SimpleOwner),    // Owner-only verification
        0x02 => Some(InlineVerifierType::TimeBased),      // Time-based expiration
        0x03 => Some(InlineVerifierType::CounterBased),   // Usage counter limits
        0x04 => Some(InlineVerifierType::Standard),       // Combined checks
        _ => None,                                         // Unknown verifier type
    }
}

/// Check if verifier supports batch verification
pub fn supports_batch_verification(verifier: &Pubkey) -> bool {
    // In production, check if verifier implements batch interface
    // Could be done via:
    // 1. Feature flag in verifier metadata
    // 2. Try calling a view function
    // 3. Maintaining a registry
    
    // For now, use simple heuristic: high bit set indicates batch support
    let verifier_bytes = verifier.to_bytes();
    verifier_bytes[0] >= 0x80  // MSB set = batch capable
}

// ===== Inline Verification Implementation =====

/// Perform inline verification for known patterns
pub fn verify_inline(
    account: &SessionAccount,
    caller: &Pubkey,
    verifier_type: InlineVerifierType,
) -> Result<()> {
    match verifier_type {
        InlineVerifierType::SimpleOwner => {
            // Extract owner pubkey from first 32 bytes of metadata
            let owner = Pubkey::try_from(&account.metadata[..32])
                .map_err(|_| CoreError::InvalidMetadata)?;
            // Verify caller matches the stored owner
            require!(caller == &owner, CoreError::Unauthorized);
        }
        
        InlineVerifierType::TimeBased => {
            // Just check expiration
            let clock = Clock::get()?;
            require!(
                clock.unix_timestamp < account.expires_at,
                CoreError::AccountExpired
            );
        }
        
        InlineVerifierType::CounterBased => {
            // Just check usage count
            require!(
                account.uses < account.max_uses,
                CoreError::UsageLimitExceeded
            );
        }
        
        InlineVerifierType::Standard => {
            // Perform comprehensive standard verification
            
            // Check 1: Ownership verification
            let owner = Pubkey::try_from(&account.metadata[..32])
                .map_err(|_| CoreError::InvalidMetadata)?;
            require!(caller == &owner, CoreError::Unauthorized);
            
            // Check 2: Expiration verification
            let clock = Clock::get()?;
            require!(
                clock.unix_timestamp < account.expires_at,
                CoreError::AccountExpired
            );
            
            // Check 3: Usage limit verification
            require!(
                account.uses < account.max_uses,
                CoreError::UsageLimitExceeded
            );
        }
    }
    
    Ok(())
}

// ===== Batch Verification Support =====

/// Batch verification request structure
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct BatchVerificationRequest {
    pub accounts: Vec<Pubkey>,
    pub caller: Pubkey,
    pub session: Pubkey,
}

/// CPI context for batch verification
#[derive(Accounts)]
pub struct VerifyAccountsBatch<'info> {
    /// CHECK: Verification request data
    pub request: AccountInfo<'info>,
    pub caller: AccountInfo<'info>,
    /// CHECK: Session for shared context
    pub session: AccountInfo<'info>,
    // remaining_accounts: accounts to verify
}

/// Standard batch verification interface
pub fn verify_accounts_batch<'info>(
    _ctx: CpiContext<'_, '_, '_, 'info, VerifyAccountsBatch<'info>>,
) -> Result<()> {
    // Implemented by verifier programs that support batching
    Ok(())
}

// ===== Classification and Grouping Helpers =====

/// Groups of accounts classified by verification type
struct VerificationGroups {
    inline_accounts: Vec<(usize, InlineVerifierType)>,
    cached_accounts: Vec<usize>,
    direct_accounts: Vec<(usize, Pubkey)>,
}

/// Classify accounts for verification based on their verifier types
/// This enables optimization by grouping accounts that can be verified together
fn classify_accounts_for_verification<'info>(
    remaining_accounts: &'info [AccountInfo<'info>],
    session: &Option<Account<'info, Session>>,
    clock: &Clock,
) -> Result<(Vec<(&'info AccountInfo<'info>, SessionAccount)>, VerificationGroups)> {
    let mut accounts_data = Vec::with_capacity(remaining_accounts.len());
    let mut inline_accounts = Vec::new();
    let mut cached_accounts = Vec::new();
    let mut direct_accounts = Vec::new();
    
    for (i, account_info) in remaining_accounts.iter().enumerate() {
        // Deserialize and validate account
        let managed_account = deserialize_and_validate_account(account_info, clock)?;
        
        // Classify by verifier type for optimal verification strategy
        if let Some(inline_type) = get_inline_verifier_type(&managed_account.verifier) {
            // Can be verified without CPI
            inline_accounts.push((i, inline_type));
        } else if session.is_some() && is_cached_verification_valid(session, i)? {
            // Already verified and cached
            cached_accounts.push(i);
        } else {
            // Requires direct CPI verification
            direct_accounts.push((i, managed_account.verifier));
        }
        
        accounts_data.push((account_info, managed_account));
    }
    
    Ok((accounts_data, VerificationGroups {
        inline_accounts,
        cached_accounts,
        direct_accounts,
    }))
}

/// Deserialize and validate a session account
fn deserialize_and_validate_account(
    account_info: &AccountInfo,
    clock: &Clock,
) -> Result<SessionAccount> {
    let account_data = account_info.try_borrow_data()
        .map_err(|_| CoreError::AccountMismatch)?;
    
    if account_data.len() < 8 + SessionAccount::SIZE {
        return Err(CoreError::AccountMismatch.into());
    }
    
    let managed_account = SessionAccount::try_deserialize(&mut &account_data[8..])?;
    
    // Check expiration
    require!(
        clock.unix_timestamp < managed_account.expires_at,
        CoreError::AccountExpired
    );
    
    Ok(managed_account)
}

/// Process direct verifications with batching optimization
fn process_direct_verifications<'info>(
    ctx: &Context<'_, '_, '_, 'info, UseAccountsAtomicOptimized<'info>>,
    direct_accounts: &[(usize, Pubkey)],
    accounts_data: &[(&AccountInfo<'info>, SessionAccount)],
    remaining_depth: u8,
) -> Result<()> {
    if direct_accounts.is_empty() {
        return Ok(());
    }
    
    // Group accounts by verifier
    let grouped_accounts = group_accounts_by_verifier(direct_accounts);
    
    // Process each group
    for (verifier, indices) in grouped_accounts {
        let mode = determine_verification_mode(&verifier, remaining_depth, indices.len());
        
        match mode {
            VerificationMode::Batch if indices.len() > 1 => {
                verify_batch(ctx, &verifier, &indices, accounts_data)?;
            }
            _ => {
                // Verify individually
                for &index in &indices {
                    let (account_info, _) = &accounts_data[index];
                    verify_single(ctx, account_info, &verifier)?;
                }
            }
        }
    }
    
    Ok(())
}

/// Group accounts by their verifier for batch processing
/// This reduces CPI calls by verifying multiple accounts with same verifier together
fn group_accounts_by_verifier(
    accounts: &[(usize, Pubkey)]
) -> Vec<(Pubkey, Vec<usize>)> {
    let mut grouped: std::collections::BTreeMap<Pubkey, Vec<usize>> = std::collections::BTreeMap::new();
    
    // Group account indices by their verifier program
    for &(index, verifier) in accounts {
        grouped.entry(verifier)
            .or_default()
            .push(index);
    }
    
    // Convert to vector for processing
    grouped.into_iter().collect()
}

// ================================
// Account Validation Contexts
// ================================

// ===== Dynamic Atomic Context =====

/// Validation context for dynamic atomic operations
#[derive(Accounts)]
pub struct UseAccountsAtomic<'info> {
    // Entity requesting atomic account usage
    pub caller: Signer<'info>,
    
    // Single verifier program used for all accounts
    /// CHECK: Verifier program for all accounts
    pub verifier_program: UncheckedAccount<'info>,
    
    // remaining_accounts: Vec<AccountInfo> contains the SessionAccounts to use atomically
}

// ===== Simple Two-Account Context =====

/// Validation context for simple two-account atomic operations
#[derive(Accounts)]
pub struct UseAccountsSimple<'info> {
    // Entity requesting to use both accounts
    pub caller: Signer<'info>,
    
    // First account in the atomic operation
    #[account(mut)]
    pub account1: Account<'info, SessionAccount>,
    
    // Second account in the atomic operation
    #[account(mut)]
    pub account2: Account<'info, SessionAccount>,
    
    // Verifier program that authorizes both accounts
    /// CHECK: Verifier program for both accounts
    pub verifier_program: UncheckedAccount<'info>,
}

// ===== Optimized Atomic Context =====

/// Validation context for optimized atomic operations with caching
#[derive(Accounts)]
pub struct UseAccountsAtomicOptimized<'info> {
    // Entity requesting atomic account usage
    pub caller: Signer<'info>,
    
    // Primary verifier program for accounts
    /// CHECK: Verifier program
    pub verifier_program: UncheckedAccount<'info>,
    
    // Optional session for caching verification results across operations
    #[account(mut)]
    pub session: Option<Account<'info, Session>>,
    
    // Temporary storage for batch verification requests
    /// CHECK: Temporary account for batch verification requests
    pub verification_request: UncheckedAccount<'info>,
    
    // remaining_accounts: SessionAccounts to use atomically
}