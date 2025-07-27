// ================================
// Atomic Operations Example
// ================================

use anchor_lang::prelude::*;
use valence_sdk::{
    session, duration, AtomicBuilder, VerificationStrategy,
    CpiDepthTracker, benchmark_strategies, PoolUpdate,
};

// ================================
// Example: DeFi Protocol with Atomic Operations
// ================================

#[derive(Debug, Clone)]
pub struct DeFiProtocolState {
    pub pools: Vec<PoolState>,
    pub total_value_locked: u64,
}

#[derive(Debug, Clone)]
pub struct PoolState {
    pub id: Pubkey,
    pub liquidity: u64,
    pub fee_bps: u16,
}

pub async fn defi_atomic_operations() -> Result<()> {
    // Create session
    let mut session = session::<DeFiProtocolState>()
        .for_entity(Pubkey::new_unique())
        .with_state(Pubkey::new_unique())
        .expires_in(duration::hours(1))
        .build()?;
    
    // ===== Simple Atomic Operation =====
    
    // Transfer between two accounts atomically
    let alice = Pubkey::new_unique();
    let bob = Pubkey::new_unique();
    
    let result = session.atomic()
        .transfer(alice, bob, 1_000_000)
        .transfer(bob, alice, 500_000)
        .execute()
        .await?;
    
    println!("Simple atomic transfer completed:");
    println!("  Operations: {}", result.operations_executed);
    println!("  Compute units: {}", result.compute_units_used);
    
    // ===== Complex Multi-Account Operation =====
    
    let pool1 = Pubkey::new_unique();
    let pool2 = Pubkey::new_unique();
    let treasury = Pubkey::new_unique();
    
    let complex_result = session.atomic()
        // Update multiple pools
        .update_pool(pool1, PoolUpdate {
            new_liquidity: Some(10_000_000),
            new_fee_bps: Some(30),
            new_owner: None,
        })
        .update_pool(pool2, PoolUpdate {
            new_liquidity: Some(5_000_000),
            new_fee_bps: None,
            new_owner: None,
        })
        // Transfer fees to treasury
        .transfer(pool1, treasury, 100_000)
        .transfer(pool2, treasury, 50_000)
        // Mint rewards
        .mint(alice, 1_000)
        .mint(bob, 500)
        .execute()
        .await?;
    
    println!("\nComplex atomic operation completed:");
    println!("  Operations: {}", complex_result.operations_executed);
    println!("  Accounts modified: {}", complex_result.accounts_modified);
    println!("  Verification mode: {:?}", complex_result.verification_mode);
    
    // ===== Optimized Atomic Operation =====
    
    // Use specific verification strategy for optimization
    let optimized_result = session.atomic()
        .with_strategy(VerificationStrategy::Batch)
        .optimize_compute()
        .transfer(alice, bob, 100_000)
        .update_pool(pool1, PoolUpdate {
            new_liquidity: Some(20_000_000),
            new_fee_bps: None,
            new_owner: None,
        })
        .execute()
        .await?;
    
    println!("\nOptimized atomic operation:");
    println!("  Compute units: {}", optimized_result.compute_units_used);
    println!("  Cached verifications: {}", optimized_result.cached_verifications);
    
    Ok(())
}

// ================================
// Example: Atomic Swap
// ================================

pub async fn atomic_swap_example() -> Result<()> {
    let mut session = session::<()>()
        .for_entity(Pubkey::new_unique())
        .expires_in(duration::minutes(5))
        .build()?;
    
    // Parties in the swap
    let alice_token_a = Pubkey::new_unique();
    let alice_token_b = Pubkey::new_unique();
    let bob_token_a = Pubkey::new_unique();
    let bob_token_b = Pubkey::new_unique();
    
    // Atomic swap: Alice gives 100 A for Bob's 50 B
    let swap_result = session.atomic()
        .transfer(alice_token_a, bob_token_a, 100)
        .transfer(bob_token_b, alice_token_b, 50)
        .with_retries(2) // Retry up to 2 times on failure
        .execute()
        .await?;
    
    println!("Atomic swap completed:");
    println!("  Execution time: {}ms", swap_result.execution_time_ms);
    
    Ok(())
}

// ================================
// Example: CPI Depth Optimization
// ================================

pub async fn cpi_optimization_example() -> Result<()> {
    // Create depth tracker
    let mut tracker = CpiDepthTracker::new();
    
    println!("Available CPI depth: {}", tracker.available_depth());
    
    // Simulate nested calls
    {
        let _guard1 = tracker.enter_cpi()?;
        println!("Depth after first CPI: {}", tracker.available_depth());
        
        {
            let _guard2 = tracker.enter_cpi()?;
            println!("Depth after second CPI: {}", tracker.available_depth());
            
            // At depth 2, we should use inline verification
            let suggestions = tracker.suggest_optimizations();
            for suggestion in suggestions {
                println!("Optimization suggestion: {:?}", suggestion);
            }
        }
    }
    
    // Record access patterns
    let accounts = vec![
        Pubkey::new_unique(),
        Pubkey::new_unique(),
        Pubkey::new_unique(),
    ];
    
    for _ in 0..5 {
        for account in &accounts {
            tracker.record_access(*account, AccessType::Verify);
        }
    }
    
    // Get optimization suggestions based on patterns
    let pattern_suggestions = tracker.suggest_optimizations();
    println!("\nPattern-based suggestions:");
    for suggestion in pattern_suggestions {
        println!("  {:?}", suggestion);
    }
    
    Ok(())
}

// ================================
// Example: Performance Comparison
// ================================

pub async fn performance_comparison() -> Result<()> {
    println!("Benchmarking verification strategies...\n");
    
    // Test with different account counts
    for account_count in [5, 10, 20, 50] {
        println!("Account count: {}", account_count);
        
        let results = benchmark_strategies(account_count).await?;
        
        for result in results {
            println!("  {} Strategy:", result.strategy);
            println!("    Compute units: {}", result.compute_units);
            println!("    CPI depth used: {}", result.cpi_depth_used);
            println!("    Cache hits: {}", result.cache_hits);
            println!("    Execution time: {}ms", result.execution_time_ms);
            println!();
        }
    }
    
    Ok(())
}

// ================================
// Example: Simulation Before Execution
// ================================

pub async fn simulation_example() -> Result<()> {
    let mut session = session::<DeFiProtocolState>()
        .for_entity(Pubkey::new_unique())
        .with_state(Pubkey::new_unique())
        .expires_in(duration::hours(1))
        .build()?;
    
    // Build complex operation
    let operation = session.atomic()
        .transfer(Pubkey::new_unique(), Pubkey::new_unique(), 1_000_000)
        .update_pool(Pubkey::new_unique(), PoolUpdate {
            new_liquidity: Some(50_000_000),
            new_fee_bps: Some(25),
            new_owner: None,
        })
        .mint(Pubkey::new_unique(), 10_000)
        .burn(Pubkey::new_unique(), 5_000)
        .with_priority_fee(1000);
    
    // Simulate first
    let simulation = operation.simulate().await?;
    
    println!("Simulation results:");
    println!("  Total compute units: {}", simulation.total_compute_units);
    println!("  Verification strategy: {:?}", simulation.verification_strategy);
    println!("  Account count: {}", simulation.account_count);
    println!("  Operation count: {}", simulation.operation_count);
    println!("  Estimated fee: {} lamports", simulation.estimated_fee);
    
    // Decide whether to execute based on simulation
    if simulation.total_compute_units < 200_000 {
        println!("\nSimulation passed, executing...");
        // In real code, would create operation again and execute
        // let result = operation.execute().await?;
    } else {
        println!("\nOperation too expensive, consider optimization");
    }
    
    Ok(())
}

// ================================
// Tests
// ================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_atomic_operations() {
        defi_atomic_operations().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_atomic_swap() {
        atomic_swap_example().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_cpi_optimization() {
        cpi_optimization_example().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_performance_comparison() {
        performance_comparison().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_simulation() {
        simulation_example().await.unwrap();
    }
}

// Re-import for examples
use valence_sdk::AccessType;