//! Orchestrates linear progression of sessions

use anyhow::Result;
use anchor_client::Client;
use tracing::{info, error, debug};
use crate::{config::Config, db::DbPool, types::*};
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;

pub async fn run_orchestrator(
    config: Config,
    db_pool: DbPool,
    anchor_client: Arc<Client>,
) -> Result<()> {
    info!("Starting lifecycle orchestrator");
    
    if !config.auto_progress {
        info!("Auto-progression disabled, orchestrator will only monitor");
    }
    
    let mut interval = tokio::time::interval(
        tokio::time::Duration::from_secs(config.poll_interval * 2) // Run less frequently
    );
    
    loop {
        interval.tick().await;
        
        // Get active progression rules
        let rules = crate::db::get_active_progression_rules(&db_pool).await?;
        
        // Process each rule
        for rule in rules {
            if let Err(e) = process_progression_rule(&config, &db_pool, &anchor_client, &rule).await {
                error!("Error processing rule {}: {}", rule.id, e);
            }
        }
        
        // Check for stale sessions
        if config.auto_progress {
            if let Err(e) = handle_stale_sessions(&config, &db_pool, &anchor_client).await {
                error!("Error handling stale sessions: {}", e);
            }
        }
    }
}

async fn process_progression_rule(
    config: &Config,
    db_pool: &DbPool,
    anchor_client: &Client,
    rule: &ProgressionRule,
) -> Result<()> {
    debug!("Processing progression rule: {}", rule.name);
    
    // Get active sessions
    let sessions = crate::db::get_active_sessions(db_pool).await?;
    
    for session in sessions {
        // Check if rule condition applies
        if check_condition(&rule.condition, &session, db_pool).await? {
            info!("Rule {} matches session {}", rule.name, session.id);
            
            if config.auto_progress {
                // Execute the action
                execute_action(&rule.action, &session, config, db_pool, anchor_client).await?;
            } else {
                // Just log what would happen
                info!("Would execute action for rule {} on session {} (auto-progress disabled)", 
                    rule.name, session.id);
            }
        }
    }
    
    Ok(())
}

async fn check_condition(
    condition: &ProgressionCondition,
    session: &Session,
    db_pool: &DbPool,
) -> Result<bool> {
    match condition {
        ProgressionCondition::AllAccountsHaveCapability(cap) => {
            // Check if all accounts in the session have the capability
            // Would need to fetch account data from chain
            debug!("Checking capability {} for session {}", cap, session.id);
            Ok(false) // Placeholder
        }
        
        ProgressionCondition::SessionIdleFor(duration) => {
            // Check if session has been idle for duration
            let now = chrono::Utc::now().timestamp();
            let idle_time = now - session.created_at;
            Ok(idle_time > *duration as i64)
        }
        
        ProgressionCondition::StateHashMatches(hash) => {
            // Would check aggregate state hash
            Ok(false) // Placeholder
        }
        
        ProgressionCondition::CustomPredicate(predicate) => {
            // Would evaluate custom predicate
            debug!("Evaluating custom predicate: {}", predicate);
            Ok(false) // Placeholder
        }
    }
}

async fn execute_action(
    action: &ProgressionAction,
    session: &Session,
    config: &Config,
    db_pool: &DbPool,
    anchor_client: &Client,
) -> Result<()> {
    match action {
        ProgressionAction::ConsumeAndCreate(templates) => {
            info!("Consuming session {} to create {} new sessions", 
                session.id, templates.len());
            
            // Build new session data from templates
            let new_sessions_data: Vec<(Vec<Pubkey>, String, u64, Vec<u8>)> = 
                templates.iter().map(|template| {
                    let accounts = template.account_indices.iter()
                        .filter_map(|&idx| session.accounts.get(idx))
                        .cloned()
                        .collect();
                    
                    (
                        accounts,
                        template.namespace.clone(),
                        rand::random::<u64>(), // nonce
                        template.metadata.clone(),
                    )
                }).collect();
            
            // Consume session on-chain
            let program = anchor_client.program(config.shard_program_id)?;
            
            let tx = program
                .request()
                .accounts(shard_accounts::ConsumeSession {
                    owner: program.payer(),
                    old_session: session.id,
                    consumption_record: Pubkey::new_unique(), // Would derive
                    instructions_sysvar: solana_sdk::sysvar::instructions::id(),
                    system_program: solana_sdk::system_program::id(),
                })
                .args(shard_instruction::ConsumeSession {
                    new_sessions_data,
                })
                .send()?;
            
            info!("Session {} consumed in tx {}", session.id, tx);
        }
        
        ProgressionAction::ExecuteBundle(template) => {
            info!("Executing bundle on session {}", session.id);
            
            // Build bundle from template
            let operations = template.operations.iter()
                .map(|op| Operation {
                    function_hash: op.function_hash,
                    args: build_args_from_template(&op.args_template, session)?,
                    expected_diff: None,
                    target_account: session.accounts.first().cloned(),
                })
                .collect::<Result<Vec<_>>>()?;
            
            let bundle = Bundle {
                operations,
                mode: ExecutionMode::Sync,
                session: session.id,
            };
            
            // Execute bundle
            let program = anchor_client.program(config.shard_program_id)?;
            
            let tx = program
                .request()
                .accounts(shard_accounts::ExecuteSyncBundle {
                    executor: program.payer(),
                    session: session.id,
                    shard_config: Pubkey::new_unique(), // Would derive
                })
                .args(shard_instruction::ExecuteSyncBundle { bundle })
                .send()?;
            
            info!("Bundle executed on session {} in tx {}", session.id, tx);
        }
        
        ProgressionAction::NotifyWebhook(url) => {
            info!("Notifying webhook {} about session {}", url, session.id);
            
            // Send webhook notification
            let client = reqwest::Client::new();
            let payload = serde_json::json!({
                "event": "session_progression",
                "session_id": session.id.to_string(),
                "accounts": session.accounts.iter().map(|a| a.to_string()).collect::<Vec<_>>(),
                "namespace": session.namespace,
            });
            
            client.post(url)
                .json(&payload)
                .send()
                .await?;
        }
    }
    
    Ok(())
}

async fn handle_stale_sessions(
    config: &Config,
    db_pool: &DbPool,
    anchor_client: &Client,
) -> Result<()> {
    debug!("Checking for stale sessions");
    
    let sessions = crate::db::get_active_sessions(db_pool).await?;
    let now = chrono::Utc::now().timestamp();
    
    for session in sessions {
        let age = now - session.created_at;
        
        if age > config.consumption_timeout as i64 {
            info!("Session {} is stale (age: {}s), marking for consumption", 
                session.id, age);
            
            // Add pending operation
            let progression_id = format!("session:{}", session.id);
            if let Some(mut progression) = crate::db::get_linear_progression(db_pool, &progression_id).await? {
                let pending_op = PendingOperation {
                    operation_type: OperationType::ConsumeSession,
                    target: session.id,
                    created_at: now,
                    expires_at: now + 3600, // 1 hour to complete
                };
                
                progression.pending_operations.push(pending_op);
                crate::db::store_linear_progression(db_pool, &progression).await?;
            }
        }
    }
    
    Ok(())
}

fn build_args_from_template(template: &str, session: &Session) -> Result<Vec<u8>> {
    // Simple template replacement
    let args_str = template
        .replace("{session_id}", &session.id.to_string())
        .replace("{namespace}", &session.namespace)
        .replace("{account_count}", &session.accounts.len().to_string());
    
    // Would parse and encode properly
    Ok(args_str.into_bytes())
}

// Mock structures
#[derive(Clone, Copy, PartialEq)]
enum ExecutionMode {
    Sync,
    Async,
}

struct Bundle {
    operations: Vec<Operation>,
    mode: ExecutionMode,
    session: Pubkey,
}

struct Operation {
    function_hash: [u8; 32],
    args: Vec<u8>,
    expected_diff: Option<[u8; 32]>,
    target_account: Option<Pubkey>,
}

mod shard_accounts {
    use anchor_lang::prelude::*;
    
    #[derive(Accounts)]
    pub struct ConsumeSession<'info> {
        pub owner: Signer<'info>,
        pub old_session: UncheckedAccount<'info>,
        pub consumption_record: UncheckedAccount<'info>,
        pub instructions_sysvar: UncheckedAccount<'info>,
        pub system_program: Program<'info, System>,
    }
    
    #[derive(Accounts)]
    pub struct ExecuteSyncBundle<'info> {
        pub executor: Signer<'info>,
        pub session: UncheckedAccount<'info>,
        pub shard_config: UncheckedAccount<'info>,
    }
}

mod shard_instruction {
    use super::*;
    use anchor_lang::prelude::*;
    
    #[derive(AnchorSerialize, AnchorDeserialize)]
    pub struct ConsumeSession {
        pub new_sessions_data: Vec<(Vec<Pubkey>, String, u64, Vec<u8>)>,
    }
    
    #[derive(AnchorSerialize, AnchorDeserialize)]
    pub struct ExecuteSyncBundle {
        pub bundle: super::Bundle,
    }
}