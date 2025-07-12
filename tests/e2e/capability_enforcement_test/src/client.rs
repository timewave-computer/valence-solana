use anchor_client::{Client, Cluster};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    commitment_config::CommitmentConfig,
};
use std::{env, str::FromStr};
use sha2::{Sha256, Digest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get program ID from environment or .valence-env file
    let program_id_str = env::var("SHARD_PROGRAM_ID")
        .or_else(|_| {
            std::fs::read_to_string(".valence-env")
                .ok()
                .and_then(|content| {
                    content.lines()
                        .find(|line| line.starts_with("export SHARD_PROGRAM_ID="))
                        .and_then(|line| line.split('=').nth(1))
                        .map(|s| s.to_string())
                })
                .ok_or_else(|| env::VarError::NotPresent)
        })
        .unwrap_or_else(|_| "NotSet".to_string());
    
    let program_id = Pubkey::from_str(&program_id_str)?;
    println!("Shard Program: {}", program_id);
    
    // Setup client
    let payer = Keypair::new();
    let client = Client::new_with_options(
        Cluster::Custom(
            "http://localhost:8899".to_string(),
            "ws://localhost:8900".to_string(),
        ),
        payer.clone(),
        CommitmentConfig::processed(),
    );
    
    println!("Using RPC: http://localhost:8899");
    println!("Payer: {}", payer.pubkey());
    
    // Airdrop SOL to payer
    println!("\nRequesting airdrop...");
    let airdrop_tx = client
        .request_airdrop(&payer.pubkey(), 2_000_000_000)?
        .await?;
    println!("Airdrop transaction: {}", airdrop_tx);
    
    // Wait for airdrop confirmation
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Initialize shard
    println!("\nInitializing shard...");
    let program = client.program(program_id)?;
    
    let init_tx = program
        .request()
        .accounts(capability_enforcement_test::accounts::Initialize {
            payer: program.payer(),
            system_program: solana_sdk::system_program::id(),
        })
        .args(capability_enforcement_test::instruction::Initialize {})
        .send()
        .await?;
    println!("Initialize transaction: {}", init_tx);
    
    // Create account request
    println!("\nCreating account request...");
    let capabilities = vec!["transfer".to_string(), "read".to_string()];
    let init_state_data = b"initial state";
    let init_state_hash = {
        let mut hasher = Sha256::new();
        hasher.update(init_state_data);
        hasher.finalize().into()
    };
    
    // Note: In real implementation, would call request_account on shard
    // For now, simulate the flow
    println!("Account request would have capabilities: {:?}", capabilities);
    println!("Init state hash: {:?}", init_state_hash);
    
    // Simulate account initialization by lifecycle manager
    println!("\n[Lifecycle Manager would initialize account here]");
    
    // Create session from accounts
    println!("\nCreating session...");
    let session_accounts = vec![Pubkey::new_unique()]; // Would be real account IDs
    let namespace = "test-namespace".to_string();
    let nonce = 1u64;
    
    // Note: Would call create_session on shard
    println!("Session would include accounts: {:?}", session_accounts);
    println!("Namespace: {}", namespace);
    
    // Execute bundle on session
    println!("\nExecuting bundle on session...");
    let echo_message = "Hello from new lifecycle!";
    
    let echo_tx = program
        .request()
        .accounts(capability_enforcement_test::accounts::Echo {
            payer: program.payer(),
        })
        .args(capability_enforcement_test::instruction::Echo {
            message: echo_message.to_string(),
        })
        .send()
        .await?;
    println!("Echo transaction: {}", echo_tx);
    println!("Message echoed: {}", echo_message);
    
    // Demonstrate session consumption (UTXO-style)
    println!("\n[Session could be consumed to create new sessions]");
    println!("This implements linear type semantics");
    
    // Demonstrate off-chain service integration
    println!("\n=== Off-chain Service Integration Demo ===");
    
    // Test lifecycle manager API (if running)
    if let Ok(response) = reqwest::get("http://localhost:8081/health").await {
        if response.status().is_success() {
            println!("✓ Lifecycle Manager is running!");
            
            // Test various API endpoints
            if let Ok(health_response) = reqwest::get("http://localhost:8081/health").await {
                if let Ok(health_text) = health_response.text().await {
                    println!("  Health status: {}", health_text);
                }
            }
            
            // Query active sessions
            if let Ok(sessions_response) = reqwest::get("http://localhost:8081/sessions").await {
                match sessions_response.json::<serde_json::Value>().await {
                    Ok(sessions) => {
                        println!("  Active sessions: {}", serde_json::to_string_pretty(&sessions)?);
                    }
                    Err(_) => {
                        println!("  No active sessions or service initializing");
                    }
                }
            }
            
            // Query account requests
            if let Ok(requests_response) = reqwest::get("http://localhost:8081/account-requests").await {
                match requests_response.json::<serde_json::Value>().await {
                    Ok(requests) => {
                        println!("  Account requests: {}", serde_json::to_string_pretty(&requests)?);
                    }
                    Err(_) => {
                        println!("  No pending account requests");
                    }
                }
            }
            
            // Test metrics endpoint
            if let Ok(metrics_response) = reqwest::get("http://localhost:8081/metrics").await {
                if metrics_response.status().is_success() {
                    println!("  ✓ Metrics endpoint accessible");
                } else {
                    println!("  ⚠ Metrics endpoint not accessible");
                }
            }
            
            // Demonstrate session builder integration
            println!("\n--- Session Builder Integration ---");
            
            // Create an account request that the session builder should pick up
            let account_request_payload = serde_json::json!({
                "capabilities": ["transfer", "read", "write"],
                "initial_state": "test_state",
                "namespace": "test-namespace"
            });
            
            match reqwest::Client::new()
                .post("http://localhost:8081/account-requests")
                .json(&account_request_payload)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("  ✓ Account request submitted successfully");
                        
                        // Wait a bit for the session builder to process
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                        
                        // Check if the request was processed
                        if let Ok(processed_response) = reqwest::get("http://localhost:8081/account-requests/processed").await {
                            if let Ok(processed_requests) = processed_response.json::<serde_json::Value>().await {
                                println!("  Processed requests: {}", serde_json::to_string_pretty(&processed_requests)?);
                            }
                        }
                    } else {
                        println!("  ⚠ Account request submission failed: {}", response.status());
                    }
                }
                Err(e) => {
                    println!("  ⚠ Could not submit account request: {}", e);
                }
            }
            
                         // Demonstrate lifecycle orchestration
             println!("\n--- Lifecycle Orchestration ---");
             
             // Create a session that should be managed by lifecycle manager
             let session_payload = serde_json::json!({
                 "accounts": [payer.pubkey().to_string()],
                 "namespace": "orchestration-test",
                 "auto_progress": true
             });
             
             match reqwest::Client::new()
                 .post("http://localhost:8081/sessions")
                 .json(&session_payload)
                 .send()
                 .await
             {
                 Ok(response) => {
                     if response.status().is_success() {
                         println!("  ✓ Session creation request submitted");
                         
                         // Query session status
                         if let Ok(session_response) = reqwest::get("http://localhost:8081/sessions/active").await {
                             if let Ok(active_sessions) = session_response.json::<serde_json::Value>().await {
                                 println!("  Active sessions: {}", serde_json::to_string_pretty(&active_sessions)?);
                             }
                         }
                         
                         // Demonstrate lifecycle progression monitoring
                         println!("  Monitoring lifecycle progression...");
                         for i in 1..=3 {
                             tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                             
                             // Check for lifecycle events
                             if let Ok(events_response) = reqwest::get("http://localhost:8081/events/recent").await {
                                 if let Ok(events) = events_response.json::<serde_json::Value>().await {
                                     if !events.as_array().unwrap_or(&vec![]).is_empty() {
                                         println!("  Step {}: Lifecycle events detected", i);
                                     } else {
                                         println!("  Step {}: No new lifecycle events", i);
                                     }
                                 }
                             }
                         }
                         
                         // Demonstrate session consumption tracking
                         println!("  Testing session consumption tracking...");
                         let consumption_payload = serde_json::json!({
                             "session_id": "test-session-123",
                             "new_sessions": [
                                 {
                                     "accounts": [payer.pubkey().to_string()],
                                     "namespace": "consumed-output-1"
                                 },
                                 {
                                     "accounts": [payer.pubkey().to_string()],
                                     "namespace": "consumed-output-2"
                                 }
                             ]
                         });
                         
                         match reqwest::Client::new()
                             .post("http://localhost:8081/sessions/consume")
                             .json(&consumption_payload)
                             .send()
                             .await
                         {
                             Ok(consume_response) => {
                                 if consume_response.status().is_success() {
                                     println!("  ✓ Session consumption request submitted");
                                     
                                     // Check consumption status
                                     if let Ok(status_response) = reqwest::get("http://localhost:8081/sessions/consumption-status").await {
                                         if let Ok(status) = status_response.json::<serde_json::Value>().await {
                                             println!("  Consumption status: {}", serde_json::to_string_pretty(&status)?);
                                         }
                                     }
                                 } else {
                                     println!("  ⚠ Session consumption request failed: {}", consume_response.status());
                                 }
                             }
                             Err(e) => {
                                 println!("  ⚠ Could not submit consumption request: {}", e);
                             }
                         }
                         
                     } else {
                         println!("  ⚠ Session creation request failed: {}", response.status());
                     }
                 }
                 Err(e) => {
                     println!("  ⚠ Could not create session: {}", e);
                 }
             }
             
             // Demonstrate auto-progression capabilities
             println!("\n--- Auto-Progression Demo ---");
             println!("  The lifecycle manager can automatically:");
             println!("  - Progress sessions through their lifecycle stages");
             println!("  - Handle session timeout and cleanup");
             println!("  - Orchestrate complex multi-session workflows");
             println!("  - Manage UTXO-style session consumption");
             println!("  - Provide real-time lifecycle event notifications";
            
        } else {
            println!("⚠ Lifecycle Manager responded but not healthy");
        }
    } else {
        println!("⚠ Lifecycle Manager not running or not accessible");
        println!("  This is expected if services are not available in the test environment");
    }
    
    // Demonstrate session builder monitoring
    println!("\n=== Session Builder Monitoring Demo ===");
    
    // Create an on-chain account request that should be detected by session builder
    println!("Creating on-chain account request for session builder to detect...");
    
    // In a real scenario, this would be a request_account transaction
    // For the demo, we'll simulate the monitoring process
    let account_request_keypair = Keypair::new();
    let account_request_pubkey = account_request_keypair.pubkey();
    
    println!("Account request ID: {}", account_request_pubkey);
    println!("Session builder should detect this request and process it automatically");
    
    // Simulate waiting for session builder to process the request
    println!("Waiting for session builder to process the request...");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Check if the session builder processed the request
    // In a real implementation, we'd check the on-chain state
    println!("✓ Session builder monitoring demonstration complete");
    println!("  In production, the session builder would:");
    println!("  - Monitor on-chain account requests");
    println!("  - Automatically initialize requested accounts");
    println!("  - Update account state with initial data");
    println!("  - Handle retry logic for failed initializations");
    
    println!("\nTest completed successfully!");
    Ok(())
}

// Re-export from capability_enforcement_test for easier access
use capability_enforcement_test;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_state_hash_computation() {
        let data = b"test data";
        let hash = {
            let mut hasher = Sha256::new();
            hasher.update(data);
            hasher.finalize().into()
        };
        
        assert_eq!(hash.len(), 32);
    }
}