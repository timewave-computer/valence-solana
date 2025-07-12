//! Database operations for lifecycle tracking

use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use crate::types::*;
use solana_sdk::pubkey::Pubkey;

pub type DbPool = Pool<Postgres>;

pub async fn init_pool(database_url: &str) -> Result<DbPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;
    
    Ok(pool)
}

pub async fn run_migrations(pool: &DbPool) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;
    Ok(())
}

// Account operations
pub async fn store_account_request(
    pool: &DbPool,
    request: &AccountRequest,
) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO account_requests (id, owner, capabilities, init_state_hash, created_at)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (id) DO NOTHING
        "#,
        request.id.to_string(),
        request.owner.to_string(),
        &request.capabilities,
        &request.init_state_hash,
        request.created_at,
    )
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn get_pending_account_requests(pool: &DbPool) -> Result<Vec<AccountRequest>> {
    let rows = sqlx::query!(
        r#"
        SELECT id, owner, capabilities, init_state_hash, created_at
        FROM account_requests
        WHERE status = 'pending'
        ORDER BY created_at ASC
        "#
    )
    .fetch_all(pool)
    .await?;
    
    let requests = rows.into_iter()
        .map(|row| AccountRequest {
            id: row.id.parse().unwrap(),
            owner: row.owner.parse().unwrap(),
            capabilities: row.capabilities,
            init_state_hash: row.init_state_hash.try_into().unwrap(),
            created_at: row.created_at,
        })
        .collect();
    
    Ok(requests)
}

pub async fn mark_account_initialized(
    pool: &DbPool,
    request_id: &Pubkey,
    account_id: &Pubkey,
) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE account_requests 
        SET status = 'initialized', account_id = $2, updated_at = NOW()
        WHERE id = $1
        "#,
        request_id.to_string(),
        account_id.to_string(),
    )
    .execute(pool)
    .await?;
    
    Ok(())
}

// Session operations
pub async fn store_session(
    pool: &DbPool,
    session: &Session,
) -> Result<()> {
    let account_ids: Vec<String> = session.accounts.iter()
        .map(|a| a.to_string())
        .collect();
    
    sqlx::query!(
        r#"
        INSERT INTO sessions (id, owner, accounts, namespace, is_consumed, nonce, created_at, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        session.id.to_string(),
        session.owner.to_string(),
        &account_ids,
        session.namespace,
        session.is_consumed,
        session.nonce as i64,
        session.created_at,
        &session.metadata,
    )
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn get_active_sessions(pool: &DbPool) -> Result<Vec<Session>> {
    let rows = sqlx::query!(
        r#"
        SELECT id, owner, accounts, namespace, is_consumed, nonce, created_at, metadata
        FROM sessions
        WHERE is_consumed = false
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;
    
    let sessions = rows.into_iter()
        .map(|row| Session {
            id: row.id.parse().unwrap(),
            owner: row.owner.parse().unwrap(),
            accounts: row.accounts.iter()
                .map(|a| a.parse().unwrap())
                .collect(),
            namespace: row.namespace,
            is_consumed: row.is_consumed,
            nonce: row.nonce as u64,
            created_at: row.created_at,
            metadata: row.metadata,
        })
        .collect();
    
    Ok(sessions)
}

pub async fn mark_session_consumed(
    pool: &DbPool,
    session_id: &Pubkey,
    consumption: &SessionConsumption,
) -> Result<()> {
    let mut tx = pool.begin().await?;
    
    // Update session
    sqlx::query!(
        r#"
        UPDATE sessions 
        SET is_consumed = true, consumed_at = $2
        WHERE id = $1
        "#,
        session_id.to_string(),
        consumption.consumed_at,
    )
    .execute(&mut *tx)
    .await?;
    
    // Store consumption record
    let created_session_ids: Vec<String> = consumption.created_sessions.iter()
        .map(|s| s.to_string())
        .collect();
    
    sqlx::query!(
        r#"
        INSERT INTO session_consumptions (consumed_session, created_sessions, transaction_signature, consumed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        consumption.consumed_session.to_string(),
        &created_session_ids,
        &consumption.transaction_signature[..],
        consumption.consumed_at,
    )
    .execute(&mut *tx)
    .await?;
    
    tx.commit().await?;
    Ok(())
}

// Linear progression tracking
pub async fn store_linear_progression(
    pool: &DbPool,
    progression: &LinearProgression,
) -> Result<()> {
    let history_json = serde_json::to_value(&progression.history)?;
    let pending_ops_json = serde_json::to_value(&progression.pending_operations)?;
    let state_json = serde_json::to_value(&progression.current_state)?;
    
    sqlx::query!(
        r#"
        INSERT INTO linear_progressions (id, current_state, history, pending_operations)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (id) DO UPDATE
        SET current_state = $2, history = $3, pending_operations = $4, updated_at = NOW()
        "#,
        progression.id,
        state_json,
        history_json,
        pending_ops_json,
    )
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn get_linear_progression(
    pool: &DbPool,
    id: &str,
) -> Result<Option<LinearProgression>> {
    let row = sqlx::query!(
        r#"
        SELECT id, current_state, history, pending_operations
        FROM linear_progressions
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?;
    
    match row {
        Some(r) => {
            let progression = LinearProgression {
                id: r.id,
                current_state: serde_json::from_value(r.current_state)?,
                history: serde_json::from_value(r.history)?,
                pending_operations: serde_json::from_value(r.pending_operations)?,
            };
            Ok(Some(progression))
        }
        None => Ok(None),
    }
}

// Lifecycle events
pub async fn store_lifecycle_event(
    pool: &DbPool,
    event: &LifecycleEvent,
) -> Result<()> {
    let event_type = match event {
        LifecycleEvent::AccountRequested { .. } => "account_requested",
        LifecycleEvent::AccountInitialized { .. } => "account_initialized",
        LifecycleEvent::SessionCreated { .. } => "session_created",
        LifecycleEvent::SessionConsumed { .. } => "session_consumed",
        LifecycleEvent::StateTransition { .. } => "state_transition",
    };
    
    let event_data = serde_json::to_value(event)?;
    
    sqlx::query!(
        r#"
        INSERT INTO lifecycle_events (event_type, event_data, created_at)
        VALUES ($1, $2, NOW())
        "#,
        event_type,
        event_data,
    )
    .execute(pool)
    .await?;
    
    Ok(())
}

// Progression rules
pub async fn get_active_progression_rules(pool: &DbPool) -> Result<Vec<ProgressionRule>> {
    let rows = sqlx::query!(
        r#"
        SELECT id, name, condition, action, enabled
        FROM progression_rules
        WHERE enabled = true
        "#
    )
    .fetch_all(pool)
    .await?;
    
    let rules = rows.into_iter()
        .map(|row| ProgressionRule {
            id: row.id,
            name: row.name,
            condition: serde_json::from_value(row.condition).unwrap(),
            action: serde_json::from_value(row.action).unwrap(),
            enabled: row.enabled,
        })
        .collect();
    
    Ok(rules)
}