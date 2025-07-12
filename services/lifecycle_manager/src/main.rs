//! Valence Lifecycle Manager Service
//! 
//! Manages the complete lifecycle of accounts and sessions:
//! - Monitors for account requests and initializes them
//! - Tracks session state and consumption
//! - Orchestrates linear type progression
//! - Provides APIs for querying lifecycle state

mod config;
mod db;
mod monitor;
mod builder;
mod orchestrator;
mod api;
mod metrics;
mod types;

use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Valence Lifecycle Manager");

    // Load configuration
    let config = config::Config::from_env()?;
    
    // Initialize database
    let db_pool = db::init_pool(&config.database_url).await?;
    db::run_migrations(&db_pool).await?;
    
    // Initialize metrics
    let metrics_registry = metrics::init_metrics();
    
    // Create service components
    let rpc_client = solana_client::rpc_client::RpcClient::new(&config.rpc_url);
    let anchor_client = Arc::new(anchor_client::Client::new(
        anchor_client::Cluster::Custom(config.rpc_url.clone(), config.ws_url.clone()),
        config.wallet_keypair.clone(),
    ));
    
    // Start background services
    let monitor_handle = tokio::spawn(monitor::run_monitor(
        config.clone(),
        db_pool.clone(),
        Arc::clone(&anchor_client),
    ));
    
    let builder_handle = tokio::spawn(builder::run_builder(
        config.clone(),
        db_pool.clone(),
        Arc::clone(&anchor_client),
    ));
    
    let orchestrator_handle = tokio::spawn(orchestrator::run_orchestrator(
        config.clone(),
        db_pool.clone(),
        Arc::clone(&anchor_client),
    ));
    
    // Start API server
    let api_handle = tokio::spawn(api::run_api_server(
        config.clone(),
        db_pool.clone(),
        metrics_registry,
    ));
    
    // Wait for shutdown signal
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal");
        }
        res = monitor_handle => {
            error!("Monitor service exited: {:?}", res);
        }
        res = builder_handle => {
            error!("Builder service exited: {:?}", res);
        }
        res = orchestrator_handle => {
            error!("Orchestrator service exited: {:?}", res);
        }
        res = api_handle => {
            error!("API service exited: {:?}", res);
        }
    }
    
    info!("Shutting down Valence Lifecycle Manager");
    Ok(())
}