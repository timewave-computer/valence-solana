//! Monitors on-chain events and updates lifecycle state

use anyhow::Result;
use anchor_client::Client;
use solana_sdk::{
    pubkey::Pubkey,
    commitment_config::CommitmentConfig,
};
use tracing::{info, error, debug};
use crate::{config::Config, db::DbPool, types::*};
use std::sync::Arc;

pub async fn run_monitor(
    config: Config,
    db_pool: DbPool,
    anchor_client: Arc<Client>,
) -> Result<()> {
    info!("Starting lifecycle monitor");
    
    let mut interval = tokio::time::interval(
        tokio::time::Duration::from_secs(config.poll_interval)
    );
    
    loop {
        interval.tick().await;
        
        // Monitor account requests
        if let Err(e) = monitor_account_requests(&config, &db_pool, &anchor_client).await {
            error!("Error monitoring account requests: {}", e);
        }
        
        // Monitor session creations
        if let Err(e) = monitor_session_creations(&config, &db_pool, &anchor_client).await {
            error!("Error monitoring session creations: {}", e);
        }
        
        // Monitor session consumptions
        if let Err(e) = monitor_session_consumptions(&config, &db_pool, &anchor_client).await {
            error!("Error monitoring session consumptions: {}", e);
        }
        
        // Update linear progressions
        if let Err(e) = update_linear_progressions(&db_pool).await {
            error!("Error updating linear progressions: {}", e);
        }
    }
}

async fn monitor_account_requests(
    config: &Config,
    db_pool: &DbPool,
    anchor_client: &Client,
) -> Result<()> {
    debug!("Checking for new account requests");
    
    // Get all AccountRequest accounts from the shard program
    let program = anchor_client.program(config.shard_program_id)?;
    
    // Use getProgramAccounts to find all AccountRequest accounts
    let accounts = program.rpc().get_program_accounts(&config.shard_program_id)?;
    
    for (pubkey, account) in accounts {
        // Check if this is an AccountRequest (by discriminator)
        if account.data.len() >= 8 {
            let discriminator = &account.data[0..8];
            
            // AccountRequest discriminator (would need to calculate actual value)
            if is_account_request_discriminator(discriminator) {
                // Deserialize AccountRequest
                if let Ok(request) = deserialize_account_request(&account.data) {
                    // Store in database
                    crate::db::store_account_request(db_pool, &request).await?;
                    
                    // Emit lifecycle event
                    let event = LifecycleEvent::AccountRequested {
                        request_id: request.id,
                        owner: request.owner,
                        capabilities: request.capabilities.clone(),
                    };
                    crate::db::store_lifecycle_event(db_pool, &event).await?;
                    
                    info!("Found new account request: {}", request.id);
                }
            }
        }
    }
    
    Ok(())
}

async fn monitor_session_creations(
    config: &Config,
    db_pool: &DbPool,
    anchor_client: &Client,
) -> Result<()> {
    debug!("Checking for new sessions");
    
    let program = anchor_client.program(config.shard_program_id)?;
    let accounts = program.rpc().get_program_accounts(&config.shard_program_id)?;
    
    for (pubkey, account) in accounts {
        if account.data.len() >= 8 {
            let discriminator = &account.data[0..8];
            
            if is_session_discriminator(discriminator) {
                if let Ok(session) = deserialize_session(&account.data) {
                    // Store in database
                    crate::db::store_session(db_pool, &session).await?;
                    
                    // Update linear progressions for accounts
                    for account_id in &session.accounts {
                        let progression_id = format!("account:{}", account_id);
                        
                        let mut progression = crate::db::get_linear_progression(db_pool, &progression_id)
                            .await?
                            .unwrap_or_else(|| LinearProgression {
                                id: progression_id.clone(),
                                current_state: LinearState::Account { id: *account_id },
                                history: vec![],
                                pending_operations: vec![],
                            });
                        
                        // Add transition
                        let transition = LinearTransition {
                            from_state: progression.current_state.clone(),
                            to_state: LinearState::InSession {
                                account_id: *account_id,
                                session_id: session.id,
                            },
                            timestamp: session.created_at,
                            transaction_signature: None, // Would get from transaction
                        };
                        
                        progression.history.push(transition);
                        progression.current_state = LinearState::InSession {
                            account_id: *account_id,
                            session_id: session.id,
                        };
                        
                        crate::db::store_linear_progression(db_pool, &progression).await?;
                    }
                    
                    // Create session progression
                    let session_progression = LinearProgression {
                        id: format!("session:{}", session.id),
                        current_state: LinearState::ActiveSession { id: session.id },
                        history: vec![],
                        pending_operations: vec![],
                    };
                    crate::db::store_linear_progression(db_pool, &session_progression).await?;
                    
                    // Emit event
                    let event = LifecycleEvent::SessionCreated {
                        session_id: session.id,
                        accounts: session.accounts.clone(),
                    };
                    crate::db::store_lifecycle_event(db_pool, &event).await?;
                    
                    info!("Found new session: {} with {} accounts", session.id, session.accounts.len());
                }
            }
        }
    }
    
    Ok(())
}

async fn monitor_session_consumptions(
    config: &Config,
    db_pool: &DbPool,
    anchor_client: &Client,
) -> Result<()> {
    debug!("Checking for session consumptions");
    
    let program = anchor_client.program(config.shard_program_id)?;
    let accounts = program.rpc().get_program_accounts(&config.shard_program_id)?;
    
    for (pubkey, account) in accounts {
        if account.data.len() >= 8 {
            let discriminator = &account.data[0..8];
            
            if is_session_consumption_discriminator(discriminator) {
                if let Ok(consumption) = deserialize_session_consumption(&account.data) {
                    // Update database
                    crate::db::mark_session_consumed(
                        db_pool,
                        &consumption.consumed_session,
                        &consumption,
                    ).await?;
                    
                    // Update linear progression
                    let progression_id = format!("session:{}", consumption.consumed_session);
                    if let Some(mut progression) = crate::db::get_linear_progression(db_pool, &progression_id).await? {
                        let transition = LinearTransition {
                            from_state: progression.current_state.clone(),
                            to_state: LinearState::ConsumedSession {
                                id: consumption.consumed_session,
                                created_sessions: consumption.created_sessions.clone(),
                            },
                            timestamp: consumption.consumed_at,
                            transaction_signature: Some(bs58::encode(&consumption.transaction_signature).into_string()),
                        };
                        
                        progression.history.push(transition);
                        progression.current_state = LinearState::ConsumedSession {
                            id: consumption.consumed_session,
                            created_sessions: consumption.created_sessions.clone(),
                        };
                        
                        crate::db::store_linear_progression(db_pool, &progression).await?;
                    }
                    
                    // Emit event
                    let event = LifecycleEvent::SessionConsumed {
                        session_id: consumption.consumed_session,
                        new_sessions: consumption.created_sessions.clone(),
                    };
                    crate::db::store_lifecycle_event(db_pool, &event).await?;
                    
                    info!("Session consumed: {} -> {:?}", 
                        consumption.consumed_session, 
                        consumption.created_sessions
                    );
                }
            }
        }
    }
    
    Ok(())
}

async fn update_linear_progressions(db_pool: &DbPool) -> Result<()> {
    // Clean up expired pending operations
    sqlx::query!(
        r#"
        UPDATE linear_progressions
        SET pending_operations = (
            SELECT jsonb_agg(op)
            FROM jsonb_array_elements(pending_operations) op
            WHERE (op->>'expires_at')::bigint > EXTRACT(EPOCH FROM NOW())
        )
        WHERE pending_operations IS NOT NULL
        "#
    )
    .execute(db_pool)
    .await?;
    
    Ok(())
}

// Helper functions for discriminators (would need actual values)
fn is_account_request_discriminator(discriminator: &[u8]) -> bool {
    // Compare with actual AccountRequest discriminator
    discriminator == &[1, 2, 3, 4, 5, 6, 7, 8] // placeholder
}

fn is_session_discriminator(discriminator: &[u8]) -> bool {
    // Compare with actual Session discriminator
    discriminator == &[9, 10, 11, 12, 13, 14, 15, 16] // placeholder
}

fn is_session_consumption_discriminator(discriminator: &[u8]) -> bool {
    // Compare with actual SessionConsumption discriminator
    discriminator == &[17, 18, 19, 20, 21, 22, 23, 24] // placeholder
}

// Deserialization helpers (would use actual Anchor deserialization)
fn deserialize_account_request(data: &[u8]) -> Result<AccountRequest> {
    // Placeholder - would use actual deserialization
    todo!("Implement actual deserialization")
}

fn deserialize_session(data: &[u8]) -> Result<Session> {
    // Placeholder - would use actual deserialization
    todo!("Implement actual deserialization")
}

fn deserialize_session_consumption(data: &[u8]) -> Result<SessionConsumption> {
    // Placeholder - would use actual deserialization
    todo!("Implement actual deserialization")
}