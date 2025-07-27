// ================================
// Example: Lending Protocol using Protocol Pattern
// ================================

use anchor_lang::prelude::*;
use valence_protocol::*;
use valence_functions::*;

declare_id!("Lend11111111111111111111111111111111111111111");

#[program]
pub mod lending_protocol {
    use super::*;
    
    /// Initialize the lending protocol with full metadata
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        // Create protocol metadata linking to off-chain data
        let metadata = ProtocolMetadata {
            ipfs_hash: "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_string(),
            version: "1.0.0".to_string(),
            description: "Decentralized lending protocol with automated risk management".to_string(),
            repository: "https://github.com/example/lending-protocol".to_string(),
            documentation: "https://docs.example.com/lending".to_string(),
            audit_reports: vec![
                "https://audit.firm1.com/reports/lending-v1.pdf".to_string(),
                "https://audit.firm2.com/reports/lending-v1.pdf".to_string(),
            ],
        };
        
        // Initialize protocol on-chain
        valence_protocol::cpi::initialize_protocol(
            ctx.accounts.into_initialize_protocol_context(),
            metadata,
        )?;
        
        msg!("Lending protocol initialized with metadata");
        Ok(())
    }
    
    /// Register the collateral ratio guard
    pub fn register_collateral_guard(ctx: Context<RegisterGuard>) -> Result<()> {
        let guard_spec = GuardSpec {
            implementation_hash: COLLATERAL_RATIO_GUARD_HASH,
            name: "collateral_ratio_guard".to_string(),
            guard_type: GuardType::Custom,
            source_ref: SourceReference {
                repository: "https://github.com/example/lending-protocol".to_string(),
                commit: "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
                path: "src/guards/collateral_ratio.rs".to_string(),
                hash: *b"12345678901234567890123456789012",
            },
            param_schema: r#"{
                "type": "object",
                "properties": {
                    "min_ratio": { "type": "number", "minimum": 100, "maximum": 200 },
                    "liquidation_threshold": { "type": "number", "minimum": 80, "maximum": 150 }
                }
            }"#.to_string(),
            security_properties: SecurityProperties {
                can_bypass: false,
                has_time_constraints: false,
                checks_signatures: false,
                max_compute_units: 5000,
                risk_level: 3,
            },
        };
        
        valence_protocol::cpi::register_guard(
            ctx.accounts.into_register_guard_context(),
            guard_spec,
        )?;
        
        msg!("Collateral ratio guard registered");
        Ok(())
    }
    
    /// Register the liquidation guard
    pub fn register_liquidation_guard(ctx: Context<RegisterGuard>) -> Result<()> {
        let guard_spec = GuardSpec {
            implementation_hash: LIQUIDATION_GUARD_HASH,
            name: "liquidation_guard".to_string(),
            guard_type: GuardType::Composite,
            source_ref: SourceReference {
                repository: "https://github.com/example/lending-protocol".to_string(),
                commit: "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
                path: "src/guards/liquidation.rs".to_string(),
                hash: *b"98765432109876543210987654321098",
            },
            param_schema: r#"{
                "type": "object",
                "properties": {
                    "max_liquidation_bonus": { "type": "number", "maximum": 10 },
                    "min_health_factor": { "type": "number", "minimum": 0.8 }
                }
            }"#.to_string(),
            security_properties: SecurityProperties {
                can_bypass: false,
                has_time_constraints: true,
                checks_signatures: true,
                max_compute_units: 10000,
                risk_level: 5,
            },
        };
        
        valence_protocol::cpi::register_guard(
            ctx.accounts.into_register_guard_context(),
            guard_spec,
        )?;
        
        msg!("Liquidation guard registered");
        Ok(())
    }
}

// ================================
// Constants and Guards
// ================================

/// Hash of the collateral ratio guard implementation
pub const COLLATERAL_RATIO_GUARD_HASH: [u8; 32] = *b"COLL_RATIO_GUARD_V1_____________";

/// Hash of the liquidation guard implementation
pub const LIQUIDATION_GUARD_HASH: [u8; 32] = *b"LIQUIDATION_GUARD_V1____________";

// ================================
// Guard Implementations
// ================================

/// Collateral ratio guard - ensures proper collateralization
pub fn collateral_ratio_guard(
    state: &LendingPosition,
    operation: &[u8],
    env: &Environment,
) -> Result<bool> {
    // This implementation would be hashed to produce COLLATERAL_RATIO_GUARD_HASH
    let min_ratio = 150; // 150% collateralization required
    
    let collateral_value = state.collateral_amount * state.collateral_price;
    let borrow_value = state.borrow_amount * state.borrow_price;
    
    if borrow_value == 0 {
        return Ok(true); // No borrows, always valid
    }
    
    let ratio = (collateral_value * 100) / borrow_value;
    Ok(ratio >= min_ratio)
}

/// Liquidation guard - controls who can liquidate and when
pub fn liquidation_guard(
    state: &LendingPosition,
    operation: &[u8],
    env: &Environment,
) -> Result<bool> {
    // Verify the position is actually liquidatable
    let health_factor = calculate_health_factor(state);
    if health_factor >= 100 {
        return Ok(false); // Position is healthy, cannot liquidate
    }
    
    // Additional checks could include:
    // - Liquidator has sufficient balance
    // - Liquidation amount is within limits
    // - Proper incentives are provided
    
    Ok(true)
}

// ================================
// Protocol States
// ================================

#[derive(Debug, Clone)]
pub struct LendingPosition {
    pub owner: Pubkey,
    pub collateral_amount: u64,
    pub collateral_price: u64,
    pub borrow_amount: u64,
    pub borrow_price: u64,
    pub last_update: i64,
}

fn calculate_health_factor(position: &LendingPosition) -> u64 {
    if position.borrow_amount == 0 {
        return u64::MAX;
    }
    
    let collateral_value = position.collateral_amount * position.collateral_price;
    let borrow_value = position.borrow_amount * position.borrow_price;
    
    (collateral_value * 100) / borrow_value
}

// ================================
// Account Contexts
// ================================

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// Protocol specification account
    /// CHECK: Created by CPI to valence-protocol
    pub protocol: UncheckedAccount<'info>,
    
    pub protocol_program: Program<'info, valence_protocol::program::ValenceProtocol>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn into_initialize_protocol_context(&self) -> CpiContext<'_, '_, '_, 'info, valence_protocol::cpi::accounts::InitializeProtocol<'info>> {
        let cpi_program = self.protocol_program.to_account_info();
        let cpi_accounts = valence_protocol::cpi::accounts::InitializeProtocol {
            authority: self.authority.to_account_info(),
            protocol: self.protocol.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct RegisterGuard<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// Protocol specification account
    #[account(mut)]
    pub protocol: Account<'info, ProtocolSpecification>,
    
    /// Guard specification account
    /// CHECK: Created by CPI to valence-protocol
    pub guard: UncheckedAccount<'info>,
    
    pub protocol_program: Program<'info, valence_protocol::program::ValenceProtocol>,
    pub system_program: Program<'info, System>,
}

impl<'info> RegisterGuard<'info> {
    pub fn into_register_guard_context(&self) -> CpiContext<'_, '_, '_, 'info, valence_protocol::cpi::accounts::RegisterGuard<'info>> {
        let cpi_program = self.protocol_program.to_account_info();
        let cpi_accounts = valence_protocol::cpi::accounts::RegisterGuard {
            authority: self.authority.to_account_info(),
            protocol: self.protocol.to_account_info(),
            guard: self.guard.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };
        CpiContext::new(cpi_program, cpi_accounts)
    }
}