//! Session Builder Service
//! 
//! Off-chain service for creating accounts based on PDAComputedEvent emissions
//! from the Valence Protocol Account Factory.

use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tokio::signal;
use tracing::{info, error};

mod config;
mod error;
mod event_monitor;
mod event_schema;
mod metrics;
mod session_builder;

use config::Config;
use metrics::MetricsServer;
use session_builder::SessionBuilder;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enable metrics server
    #[arg(long)]
    enable_metrics: bool,
    
    /// Metrics server port
    #[arg(long, default_value = "3001")]
    metrics_port: u16,
    
    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,
    
    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize tracing
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(&args.log_level)
        .with_target(false)
        .compact()
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)?;
    
    info!("Starting Valence Session Builder Service");
    
    // Load configuration
    let config = Arc::new(Config::load(args.config.as_deref())?);
    
    // Initialize metrics server if enabled
    let metrics_server = if args.enable_metrics {
        let server = MetricsServer::new(args.metrics_port);
        server.start().await?;
        Some(server)
    } else {
        None
    };
    
    // Create valence domain client
    use valence_domain_clients::solana::SolanaRpcClient;
    let client = Arc::new(valence_domain_clients::solana::SolanaLocalnetClient::from_bytes(
        &config.load_keypair()?.to_bytes()
    )?);
    
    // Initialize session builder
    let session_builder = SessionBuilder::new(config.clone(), client).await?;
    
    // Handle graceful shutdown
    let shutdown_signal = async {
        signal::ctrl_c()
            .await
            .expect("Failed to listen for shutdown signal");
        info!("Received shutdown signal");
    };
    
    // Run the service
    tokio::select! {
        result = session_builder.run() => {
            if let Err(e) = result {
                error!("Session builder error: {}", e);
                return Err(e);
            }
        }
        _ = shutdown_signal => {
            info!("Shutting down gracefully...");
        }
    }
    
    // Cleanup
    session_builder.stop();
    
    if let Some(server) = metrics_server {
        server.shutdown().await?;
    }
    
    info!("Session Builder Service stopped");
    Ok(())
} 