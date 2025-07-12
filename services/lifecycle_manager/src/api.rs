//! REST API for lifecycle management

use anyhow::Result;
use warp::{Filter, Reply, Rejection};
use crate::{config::Config, db::DbPool, types::*};
use prometheus::Registry;
use std::sync::Arc;

pub async fn run_api_server(
    config: Config,
    db_pool: DbPool,
    metrics_registry: Registry,
) -> Result<()> {
    let db_pool = Arc::new(db_pool);
    let metrics_registry = Arc::new(metrics_registry);
    
    // Health check endpoint
    let health = warp::path("health")
        .and(warp::get())
        .map(|| warp::reply::json(&serde_json::json!({ "status": "ok" })));
    
    // Metrics endpoint
    let metrics = warp::path("metrics")
        .and(warp::get())
        .and(with_metrics(metrics_registry.clone()))
        .and_then(get_metrics);
    
    // Get account status
    let get_account = warp::path!("accounts" / String)
        .and(warp::get())
        .and(with_db(db_pool.clone()))
        .and_then(get_account_handler);
    
    // Get session status
    let get_session = warp::path!("sessions" / String)
        .and(warp::get())
        .and(with_db(db_pool.clone()))
        .and_then(get_session_handler);
    
    // Get linear progression
    let get_progression = warp::path!("progressions" / String)
        .and(warp::get())
        .and(with_db(db_pool.clone()))
        .and_then(get_progression_handler);
    
    // List active sessions
    let list_sessions = warp::path!("sessions")
        .and(warp::get())
        .and(with_db(db_pool.clone()))
        .and_then(list_sessions_handler);
    
    // Get lifecycle events
    let get_events = warp::path!("events")
        .and(warp::get())
        .and(warp::query::<EventQuery>())
        .and(with_db(db_pool.clone()))
        .and_then(get_events_handler);
    
    // Create progression rule
    let create_rule = warp::path!("rules")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db_pool.clone()))
        .and_then(create_rule_handler);
    
    // Combine all routes
    let routes = health
        .or(metrics)
        .or(get_account)
        .or(get_session)
        .or(get_progression)
        .or(list_sessions)
        .or(get_events)
        .or(create_rule)
        .with(warp::cors().allow_any_origin());
    
    tracing::info!("API server listening on port {}", config.api_port);
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], config.api_port))
        .await;
    
    Ok(())
}

// Helper filters
fn with_db(db_pool: Arc<DbPool>) -> impl Filter<Extract = (Arc<DbPool>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db_pool.clone())
}

fn with_metrics(registry: Arc<Registry>) -> impl Filter<Extract = (Arc<Registry>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || registry.clone())
}

// Handler functions
async fn get_metrics(registry: Arc<Registry>) -> Result<impl Reply, Rejection> {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    Ok(warp::reply::with_header(
        buffer,
        "Content-Type",
        encoder.format_type(),
    ))
}

async fn get_account_handler(
    account_id: String,
    db_pool: Arc<DbPool>,
) -> Result<impl Reply, Rejection> {
    let progression_id = format!("account:{}", account_id);
    let progression = crate::db::get_linear_progression(&*db_pool, &progression_id)
        .await
        .map_err(|e| warp::reject::custom(ApiError::DatabaseError(e.to_string())))?;
    
    match progression {
        Some(p) => Ok(warp::reply::json(&p)),
        None => Err(warp::reject::not_found()),
    }
}

async fn get_session_handler(
    session_id: String,
    db_pool: Arc<DbPool>,
) -> Result<impl Reply, Rejection> {
    let progression_id = format!("session:{}", session_id);
    let progression = crate::db::get_linear_progression(&*db_pool, &progression_id)
        .await
        .map_err(|e| warp::reject::custom(ApiError::DatabaseError(e.to_string())))?;
    
    match progression {
        Some(p) => Ok(warp::reply::json(&p)),
        None => Err(warp::reject::not_found()),
    }
}

async fn get_progression_handler(
    progression_id: String,
    db_pool: Arc<DbPool>,
) -> Result<impl Reply, Rejection> {
    let progression = crate::db::get_linear_progression(&*db_pool, &progression_id)
        .await
        .map_err(|e| warp::reject::custom(ApiError::DatabaseError(e.to_string())))?;
    
    match progression {
        Some(p) => Ok(warp::reply::json(&p)),
        None => Err(warp::reject::not_found()),
    }
}

async fn list_sessions_handler(
    db_pool: Arc<DbPool>,
) -> Result<impl Reply, Rejection> {
    let sessions = crate::db::get_active_sessions(&*db_pool)
        .await
        .map_err(|e| warp::reject::custom(ApiError::DatabaseError(e.to_string())))?;
    
    Ok(warp::reply::json(&sessions))
}

#[derive(serde::Deserialize)]
struct EventQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    event_type: Option<String>,
}

async fn get_events_handler(
    query: EventQuery,
    db_pool: Arc<DbPool>,
) -> Result<impl Reply, Rejection> {
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or(0);
    
    let events = if let Some(event_type) = query.event_type {
        sqlx::query!(
            r#"
            SELECT event_type, event_data, created_at
            FROM lifecycle_events
            WHERE event_type = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            event_type,
            limit,
            offset,
        )
        .fetch_all(&**db_pool)
        .await
    } else {
        sqlx::query!(
            r#"
            SELECT event_type, event_data, created_at
            FROM lifecycle_events
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset,
        )
        .fetch_all(&**db_pool)
        .await
    }
    .map_err(|e| warp::reject::custom(ApiError::DatabaseError(e.to_string())))?;
    
    let events: Vec<_> = events.into_iter()
        .map(|row| serde_json::json!({
            "event_type": row.event_type,
            "event_data": row.event_data,
            "created_at": row.created_at,
        }))
        .collect();
    
    Ok(warp::reply::json(&events))
}

async fn create_rule_handler(
    rule: ProgressionRule,
    db_pool: Arc<DbPool>,
) -> Result<impl Reply, Rejection> {
    let condition_json = serde_json::to_value(&rule.condition)
        .map_err(|e| warp::reject::custom(ApiError::InvalidInput(e.to_string())))?;
    
    let action_json = serde_json::to_value(&rule.action)
        .map_err(|e| warp::reject::custom(ApiError::InvalidInput(e.to_string())))?;
    
    sqlx::query!(
        r#"
        INSERT INTO progression_rules (id, name, condition, action, enabled)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        rule.id,
        rule.name,
        condition_json,
        action_json,
        rule.enabled,
    )
    .execute(&**db_pool)
    .await
    .map_err(|e| warp::reject::custom(ApiError::DatabaseError(e.to_string())))?;
    
    Ok(warp::reply::json(&serde_json::json!({
        "status": "created",
        "rule_id": rule.id,
    })))
}

// Error handling
#[derive(Debug)]
enum ApiError {
    DatabaseError(String),
    InvalidInput(String),
}

impl warp::reject::Reject for ApiError {}