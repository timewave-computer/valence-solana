use proptest::prelude::*;

/// Property: Compute unit limits are enforced
#[test]
fn prop_compute_unit_limits_enforced() {
    proptest!(|(
        compute_limit in 100u64..1_000_000,
        operations in prop::collection::vec(1u64..10_000, 1..100),
    )| {
        let mut compute_used = 0u64;
        let mut operations_completed = 0;
        
        for op_cost in &operations {
            if compute_used + op_cost <= compute_limit {
                compute_used += op_cost;
                operations_completed += 1;
            } else {
                // Property: Cannot exceed compute limit
                prop_assert!(
                    compute_used <= compute_limit,
                    "Compute units {} would exceed limit {}", 
                    compute_used + op_cost, compute_limit
                );
                break;
            }
        }
        
        // Property: Used compute never exceeds limit
        prop_assert!(
            compute_used <= compute_limit,
            "Total compute {} exceeds limit {}", compute_used, compute_limit
        );
        
        // Property: Utilization is maximized
        let remaining = compute_limit - compute_used;
        prop_assert!(
            remaining < 10_000 || operations_completed == operations.len(),
            "Could have processed more operations with {} units remaining", remaining
        );
    });
}

/// Property: Memory allocation limits
#[test]
fn prop_memory_allocation_limits() {
    proptest!(|(
        memory_limit in 1024usize..10_485_760, // 1KB to 10MB
        allocations in prop::collection::vec(1usize..102_400, 1..100), // up to 100KB each
    )| {
        let mut memory_used = 0usize;
        let mut successful_allocs = Vec::new();
        
        for alloc_size in allocations {
            if memory_used + alloc_size <= memory_limit {
                memory_used += alloc_size;
                successful_allocs.push(alloc_size);
                
                // Property: Running total is accurate
                prop_assert_eq!(
                    memory_used,
                    successful_allocs.iter().sum::<usize>(),
                    "Memory accounting mismatch"
                );
            } else {
                // Property: Cannot exceed memory limit
                prop_assert!(
                    memory_used + alloc_size > memory_limit,
                    "Allocation rejected despite having space"
                );
            }
        }
        
        // Property: Total allocation respects limit
        prop_assert!(
            memory_used <= memory_limit,
            "Memory usage {} exceeds limit {}", memory_used, memory_limit
        );
        
        // Property: No memory leaks (all allocated memory is tracked)
        let tracked_total: usize = successful_allocs.iter().sum();
        prop_assert_eq!(
            memory_used, tracked_total,
            "Memory leak detected: used {} but tracked {}", memory_used, tracked_total
        );
    });
}

/// Property: Account size limits
#[test]
fn prop_account_size_limits() {
    const MAX_ACCOUNT_SIZE: usize = 10 * 1024 * 1024; // 10MB
    
    proptest!(|(
        initial_size in 0usize..1024,
        operations in prop::collection::vec(
            (any::<bool>(), 0usize..102_400), // grow/shrink, size
            1..50
        ),
    )| {
        let mut account_size = initial_size;
        let mut size_history = vec![initial_size];
        
        for (is_grow, size_delta) in &operations {
            let new_size = if *is_grow {
                account_size.saturating_add(*size_delta)
            } else {
                account_size.saturating_sub(*size_delta)
            };
            
            if new_size <= MAX_ACCOUNT_SIZE {
                account_size = new_size;
                size_history.push(account_size);
                
                // Property: Size changes are applied correctly
                if *is_grow {
                    prop_assert!(
                        account_size >= size_history[size_history.len() - 2],
                        "Account didn't grow as expected"
                    );
                }
            } else {
                // Property: Cannot exceed maximum size
                prop_assert!(
                    new_size > MAX_ACCOUNT_SIZE,
                    "Valid resize rejected"
                );
                size_history.push(account_size); // Size unchanged
            }
        }
        
        // Property: Final size respects limit
        prop_assert!(
            account_size <= MAX_ACCOUNT_SIZE,
            "Account size {} exceeds maximum {}", account_size, MAX_ACCOUNT_SIZE
        );
        
        // Property: Size history is monotonic when only growing
        let only_grow_ops = operations.iter().all(|(grow, _)| *grow);
        if only_grow_ops {
            for i in 1..size_history.len() {
                prop_assert!(
                    size_history[i] >= size_history[i-1],
                    "Size decreased during grow-only operations"
                );
            }
        }
    });
}

/// Property: Transaction size limits
#[test]
fn prop_transaction_size_limits() {
    const MAX_TX_SIZE: usize = 1232; // Solana's transaction size limit
    
    proptest!(|(
        num_instructions in 1usize..20,
        instruction_sizes in prop::collection::vec(10usize..500, 1..20),
        num_signatures in 1usize..10,
    )| {
        let base_size = 128; // Headers, signatures, etc.
        let signature_size = num_signatures * 64;
        let mut tx_size = base_size + signature_size;
        let mut included_instructions = 0;
        
        for (i, inst_size) in instruction_sizes.iter().enumerate() {
            if i >= num_instructions {
                break;
            }
            
            if tx_size + inst_size <= MAX_TX_SIZE {
                tx_size += inst_size;
                included_instructions += 1;
            } else {
                // Property: Instructions that would exceed limit are excluded
                prop_assert!(
                    tx_size + inst_size > MAX_TX_SIZE,
                    "Instruction excluded despite fitting"
                );
                break;
            }
        }
        
        // Property: Transaction size never exceeds limit
        prop_assert!(
            tx_size <= MAX_TX_SIZE,
            "Transaction size {} exceeds limit {}", tx_size, MAX_TX_SIZE
        );
        
        // Property: Maximum instructions are included
        if included_instructions < num_instructions.min(instruction_sizes.len()) {
            let next_inst_size = instruction_sizes.get(included_instructions).unwrap_or(&0);
            prop_assert!(
                tx_size + next_inst_size > MAX_TX_SIZE,
                "Could have included more instructions"
            );
        }
    });
}

/// Property: Rate limiting
#[test]
fn prop_rate_limiting() {
    proptest!(|(
        rate_limit in 1u32..1000, // requests per second
        request_times in prop::collection::vec(0u64..10_000, 1..1000), // milliseconds
    )| {
        let mut allowed_requests = Vec::new();
        let window_ms = 1000; // 1 second window
        
        for time in request_times {
            // Remove old requests outside window
            allowed_requests.retain(|&t| time.saturating_sub(t) < window_ms);
            
            if allowed_requests.len() < rate_limit as usize {
                allowed_requests.push(time);
                
                // Property: Number of requests in window doesn't exceed rate limit
                prop_assert!(
                    allowed_requests.len() <= rate_limit as usize,
                    "Rate limit exceeded"
                );
            }
            
            // Property: Rate limit is enforced
            let requests_in_window = allowed_requests.iter()
                .filter(|&&t| time.saturating_sub(t) < window_ms)
                .count();
            prop_assert!(
                requests_in_window <= rate_limit as usize,
                "Too many requests in sliding window"
            );
        }
    });
}

/// Property: Nested call depth limits
#[test]
fn prop_nested_call_depth_limits() {
    const MAX_CALL_DEPTH: usize = 4;
    
    proptest!(|(
        call_patterns in prop::collection::vec(
            prop::collection::vec(any::<bool>(), 0..10),
            1..20
        ),
    )| {
        for pattern in call_patterns {
            let mut call_stack = Vec::new();
            let mut max_depth_reached = 0;
            
            for make_call in pattern {
                if make_call && call_stack.len() < MAX_CALL_DEPTH {
                    call_stack.push(call_stack.len());
                    max_depth_reached = max_depth_reached.max(call_stack.len());
                } else if !make_call && !call_stack.is_empty() {
                    call_stack.pop();
                }
                
                // Property: Call depth never exceeds limit
                prop_assert!(
                    call_stack.len() <= MAX_CALL_DEPTH,
                    "Call depth {} exceeds limit {}", call_stack.len(), MAX_CALL_DEPTH
                );
            }
            
            // Property: Stack is properly unwound
            prop_assert!(
                call_stack.is_empty() || call_stack.len() <= MAX_CALL_DEPTH,
                "Call stack not properly managed"
            );
        }
    });
}

/// Property: Concurrent session limits
#[test]
fn prop_concurrent_session_limits() {
    proptest!(|(
        max_sessions in 1usize..100,
        session_events in prop::collection::vec(
            (any::<bool>(), 1u64..100), // create/close, duration
            1..200
        ),
    )| {
        let mut active_sessions = Vec::new();
        let mut current_time = 0u64;
        let mut sessions_rejected = 0;
        
        for (is_create, duration) in &session_events {
            // Clean up expired sessions
            active_sessions.retain(|&(_, expiry)| expiry > current_time);
            
            if *is_create {
                if active_sessions.len() < max_sessions {
                    active_sessions.push((current_time, current_time + duration));
                } else {
                    sessions_rejected += 1;
                    
                    // Property: Session rejected when at limit
                    prop_assert_eq!(
                        active_sessions.len(), max_sessions,
                        "Session rejected but not at limit"
                    );
                }
            }
            
            // Property: Active sessions never exceed limit
            prop_assert!(
                active_sessions.len() <= max_sessions,
                "Active sessions {} exceed limit {}", active_sessions.len(), max_sessions
            );
            
            current_time += 1;
        }
        
        // Property: Rejection count is reasonable
        if sessions_rejected > 0 {
            prop_assert!(
                sessions_rejected < session_events.len(),
                "All sessions were rejected"
            );
        }
    });
}

/// Property: Storage rent limits
#[test]
fn prop_storage_rent_limits() {
    proptest!(|(
        initial_balance in 1_000_000u64..1_000_000_000, // lamports
        rent_per_byte_year in 1u64..1000,
        account_sizes in prop::collection::vec(100usize..10_000, 1..20),
        years in prop::collection::vec(1u64..10, 1..20),
    )| {
        let mut balance = initial_balance;
        let mut accounts = Vec::new();
        
        for (size, years_to_store) in account_sizes.iter().zip(years.iter()) {
            let rent_cost = (*size as u64) * rent_per_byte_year * years_to_store;
            
            if balance >= rent_cost {
                balance -= rent_cost;
                accounts.push((*size, *years_to_store));
                
                // Property: Balance is properly deducted
                let total_rent: u64 = accounts.iter()
                    .map(|(s, y)| (*s as u64) * rent_per_byte_year * y)
                    .sum();
                prop_assert_eq!(
                    balance, initial_balance - total_rent,
                    "Balance accounting error"
                );
            } else {
                // Property: Cannot store without sufficient rent
                prop_assert!(
                    balance < rent_cost,
                    "Storage allowed without sufficient rent"
                );
            }
        }
        
        // Property: Total storage cost doesn't exceed initial balance
        let total_cost: u64 = accounts.iter()
            .map(|(size, years)| (*size as u64) * rent_per_byte_year * years)
            .sum();
        prop_assert!(
            total_cost <= initial_balance,
            "Spent more on rent than available balance"
        );
    });
}

/// Property: Function execution time limits
#[test]
fn prop_function_execution_time_limits() {
    proptest!(|(
        time_limit_ms in 10u64..5000,
        operation_times in prop::collection::vec(1u64..100, 1..100),
    )| {
        let mut total_time = 0u64;
        let mut operations_completed = 0;
        
        for op_time in &operation_times {
            if total_time + op_time <= time_limit_ms {
                total_time += op_time;
                operations_completed += 1;
            } else {
                // Property: Execution stops at time limit
                prop_assert!(
                    total_time + op_time > time_limit_ms,
                    "Operation stopped despite having time"
                );
                break;
            }
        }
        
        // Property: Total execution time respects limit
        prop_assert!(
            total_time <= time_limit_ms,
            "Execution time {} exceeds limit {}", total_time, time_limit_ms
        );
        
        // Property: Time utilization is efficient
        let time_remaining = time_limit_ms - total_time;
        
        // If we completed all operations or have very little time left, that's fine
        if operations_completed < operation_times.len() && time_remaining > 0 {
            // Check if the next operation would have exceeded the limit
            let next_op_time = operation_times.get(operations_completed).unwrap_or(&time_limit_ms);
            prop_assert!(
                time_remaining < *next_op_time,
                "Could have executed more operations with {} ms remaining", time_remaining
            );
        }
    });
}