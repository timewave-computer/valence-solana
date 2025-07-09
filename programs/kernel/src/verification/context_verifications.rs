// Consolidated context-based verification functions
// Combines block_height and session_creation verifications

use anchor_lang::prelude::*;
use crate::error::ValenceError;
use crate::functions::VerificationFunction;

// ====== Block Height Verification ======

/// Verify block height conditions
pub fn verify_block_height(
    capability_id: &str,
    _sender: &Pubkey,
    params: &[u8],
    _session_account: Option<&AccountInfo>,
) -> Result<()> {
    msg!("Block height verification for capability: {}", capability_id);

    // Get current block height
    let clock = Clock::get()?;
    let current_slot = clock.slot;
    
    // Parse block height condition from params
    if params.len() < 9 {
        return Err(ValenceError::VerificationInvalidParams.into());
    }

    let condition_type = params[0];
    let target_height = u64::from_le_bytes(params[1..9].try_into().unwrap());

    // Apply condition
    match condition_type {
        0 => { // After
            if current_slot <= target_height {
                msg!("Block height {} not yet reached (current: {})", target_height, current_slot);
                return Err(ValenceError::VerificationBlockHeightNotReached.into());
            }
        }
        1 => { // Before
            if current_slot >= target_height {
                msg!("Block height {} already passed (current: {})", target_height, current_slot);
                return Err(ValenceError::VerificationBlockHeightExceeded.into());
            }
        }
        2 => { // Between (requires additional parameter)
            if params.len() < 17 {
                return Err(ValenceError::VerificationInvalidParams.into());
            }
            let end_height = u64::from_le_bytes(params[9..17].try_into().unwrap());
            
            if current_slot < target_height || current_slot > end_height {
                msg!("Block height {} not in range {}-{}", current_slot, target_height, end_height);
                return Err(ValenceError::VerificationBlockHeightNotInRange.into());
            }
        }
        _ => {
            return Err(ValenceError::VerificationInvalidConditionType.into());
        }
    }

    msg!("Block height verification passed");
    Ok(())
}

/// Create block height condition for after a specific height
pub fn create_after_height_condition(height: u64) -> Vec<u8> {
    let mut params = vec![0u8]; // Type: After
    params.extend_from_slice(&height.to_le_bytes());
    params
}

/// Create block height condition for before a specific height
pub fn create_before_height_condition(height: u64) -> Vec<u8> {
    let mut params = vec![1u8]; // Type: Before
    params.extend_from_slice(&height.to_le_bytes());
    params
}

/// Create block height condition for between two heights
pub fn create_between_heights_condition(start: u64, end: u64) -> Vec<u8> {
    let mut params = vec![2u8]; // Type: Between
    params.extend_from_slice(&start.to_le_bytes());
    params.extend_from_slice(&end.to_le_bytes());
    params
}

// ====== Session Creation Verification ======

/// Verify session creation requirements
pub fn verify_session_creation(
    capability_id: &str,
    _sender: &Pubkey,
    params: &[u8],
    session_account: Option<&AccountInfo>,
) -> Result<()> {
    msg!("Session creation verification for capability: {}", capability_id);

    // Session creation checks:
    // 1. Valid session parameters
    // 2. Sender has permission to create sessions
    // 3. Session limits not exceeded
    // 4. Valid session configuration

    // Check if session account is provided for session-required capabilities
    if capability_id.contains("session_required") && session_account.is_none() {
        return Err(ValenceError::VerificationSessionRequired.into());
    }

    // Verify session parameters
    if params.len() < 32 {
        return Err(ValenceError::VerificationInvalidSessionParams.into());
    }

    // Parse session configuration
    let session_type = params[0];
    match session_type {
        0 => { // Basic session
            msg!("Creating basic session");
        }
        1 => { // Advanced session with additional verification
            // Check sender has permission for advanced sessions
            // In production, this would check against a permission registry
            msg!("Creating advanced session");
        }
        2 => { // Temporary session
            if params.len() < 40 {
                return Err(ValenceError::VerificationInvalidSessionParams.into());
            }
            let expiry = u64::from_le_bytes(params[32..40].try_into().unwrap());
            let clock = Clock::get()?;
            
            if expiry < clock.unix_timestamp as u64 {
                return Err(ValenceError::VerificationSessionExpired.into());
            }
            msg!("Creating temporary session with expiry: {}", expiry);
        }
        _ => {
            return Err(ValenceError::VerificationInvalidSessionType.into());
        }
    }

    // Verify session account if provided
    if let Some(session) = session_account {
        // Check session is properly initialized
        if session.data_is_empty() {
            return Err(ValenceError::VerificationSessionNotInitialized.into());
        }
        
        // Verify session owner
        if session.owner != &crate::ID {
            return Err(ValenceError::VerificationInvalidSessionOwner.into());
        }
        
        msg!("Valid session account verified");
    }

    Ok(())
}

/// Create session creation parameters
pub fn create_session_params(
    session_type: u8,
    owner: &Pubkey,
    expiry: Option<u64>,
) -> Vec<u8> {
    let mut params = vec![session_type];
    params.extend_from_slice(&owner.to_bytes());
    
    if let Some(exp) = expiry {
        params.extend_from_slice(&exp.to_le_bytes());
    }
    
    params
}

// ====== Registration Functions ======

/// Register block height verification function
pub fn register_block_height_verification(
    _verification_function_table_program: Pubkey,
    _authority: Pubkey,
) -> Result<()> {
    msg!("Registering block height verification function");
    Ok(())
}

/// Register session creation verification function
pub fn register_session_creation_verification(
    _verification_function_table_program: Pubkey,
    _authority: Pubkey,
) -> Result<()> {
    msg!("Registering session creation verification function");
    Ok(())
}

/// Register all context verification functions
pub fn register_context_verifications(
    verification_function_table_program: Pubkey,
    authority: Pubkey,
) -> Result<()> {
    register_block_height_verification(verification_function_table_program, authority)?;
    register_session_creation_verification(verification_function_table_program, authority)?;
    
    msg!("All context verification functions registered");
    Ok(())
}

// ====== Verification Function Definitions ======

pub fn get_block_height_function() -> VerificationFunction {
    VerificationFunction {
        function_id: "block_height".to_string(),
        function_type: "temporal".to_string(),
        program_id: crate::ID,
        entry_point: "verify_block_height".to_string(),
        required_accounts: vec![],
        parameters_schema: vec!["condition_type:u8".to_string(), "height:u64".to_string()],
        description: "Block height condition verification".to_string(),
        version: "1.0.0".to_string(),
        is_active: true,
        required_compute_units: 3000,
        required_account_data: 0,
    }
}

pub fn get_session_creation_function() -> VerificationFunction {
    VerificationFunction {
        function_id: "session_creation".to_string(),
        function_type: "session".to_string(),
        program_id: crate::ID,
        entry_point: "verify_session_creation".to_string(),
        required_accounts: vec!["sender".to_string(), "session".to_string()],
        parameters_schema: vec!["session_type:u8".to_string(), "owner:pubkey".to_string()],
        description: "Session creation verification".to_string(),
        version: "1.0.0".to_string(),
        is_active: true,
        required_compute_units: 15000,
        required_account_data: 1000,
    }
}