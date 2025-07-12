//! Session Builder Service
//! 
//! Monitors on-chain session requests and initializes them with declared state

pub mod monitor;
pub mod builder;
pub mod config;

pub use monitor::SessionMonitor;
pub use builder::SessionBuilder;
pub use config::Config;

use anyhow::Result;

/// Main entry point for the session builder service
pub async fn run(config: Config) -> Result<()> {
    println!("Starting session builder service...");
    
    // Create monitor to watch for session requests
    let monitor = SessionMonitor::new(config.clone())?;
    
    // Create builder to initialize sessions
    let builder = SessionBuilder::new(config)?;
    
    // Main service loop
    loop {
        // Poll for new session requests
        match monitor.poll_requests().await {
            Ok(requests) => {
                for request in requests {
                    println!("Processing session request: {:?}", request.id);
                    
                    // Attempt to initialize the session
                    match builder.initialize_session(request).await {
                        Ok(signature) => {
                            println!("Session initialized successfully: {}", signature);
                        }
                        Err(e) => {
                            eprintln!("Failed to initialize session: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error polling requests: {}", e);
            }
        }
        
        // Wait before next poll
        tokio::time::sleep(monitor.poll_interval()).await;
    }
}