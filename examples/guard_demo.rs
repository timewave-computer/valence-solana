// Example demonstrating the complete guard system workflow
// This shows how to create sessions with guards and execute operations

use anchor_lang::prelude::*;
use valence_core::*;

/// Example program showing guard system usage
fn main() {
    println!("=== Valence Core Guard System Demo ===\n");
    
    // 1. Create basic components
    let owner = Pubkey::new_unique();
    let protocol = Pubkey::new_unique();
    let auth_state = Pubkey::new_unique();
    
    println!("Owner: {}", owner);
    println!("Protocol: {}", protocol);
    println!("Auth State: {}\n", auth_state);
    
    // 2. Create different types of guards
    demo_simple_guards();
    demo_composite_guards();
    demo_time_based_guards();
    demo_permission_guards();
    demo_usage_guards();
}

fn demo_simple_guards() {
    println!("=== Simple Guards ===");
    
    // Owner-only guard
    let owner_guard = guards::Guard::owner_only();
    println!("Owner-only guard: {:?}", owner_guard);
    
    // Always allow guard (public access)
    let public_guard = guards::Guard::allow_all();
    println!("Public access guard: {:?}", public_guard);
    
    // Always deny guard (disabled)
    let disabled_guard = guards::Guard::deny_all();
    println!("Disabled guard: {:?}\n", disabled_guard);
}

fn demo_composite_guards() {
    println!("=== Composite Guards ===");
    
    // AND combination: Owner AND not expired
    let secure_guard = guards::Guard::and(
        guards::Guard::owner_only(),
        guards::Guard::expires_at(1234567890 + 3600)
    );
    println!("Secure guard (owner AND not expired): {:?}", secure_guard);
    
    // OR combination: Owner OR has special permission
    let flexible_guard = guards::Guard::or(
        guards::Guard::owner_only(),
        guards::Guard::requires_permission(0b0001) // Admin permission
    );
    println!("Flexible guard (owner OR admin): {:?}", flexible_guard);
    
    // NOT guard: Anyone except a specific program
    let exclusion_guard = guards::Guard::not(
        guards::Guard::external_simple(Pubkey::new_unique())
    );
    println!("Exclusion guard (NOT external): {:?}\n", exclusion_guard);
}

fn demo_time_based_guards() {
    println!("=== Time-based Guards ===");
    
    let current_time = 1234567890i64;
    
    // Expires in 1 hour
    let temp_access = guards::Guard::expires_in(3600, current_time);
    println!("Temporary access (1 hour): {:?}", temp_access);
    
    // Time-locked (only accessible after certain time)
    let time_locked = guards::Guard::time_lock(current_time + 86400);
    println!("Time-locked (24 hours): {:?}", time_locked);
    
    // Business hours (9 AM to 5 PM)
    let business_hours = guards::Guard::business_hours(9, 17, current_time);
    println!("Business hours guard: {:?}\n", business_hours);
}

fn demo_permission_guards() {
    println!("=== Permission Guards ===");
    
    // Single permission
    let read_permission = guards::Guard::requires_permission(0b0001);
    println!("Read permission: {:?}", read_permission);
    
    // Multiple permissions (using bits)
    let write_permission = guards::Guard::requires_permission(0b0010);
    let admin_permission = guards::Guard::requires_permission(0b0100);
    
    // Combine permissions: Read AND Write
    let read_write = guards::Guard::and(read_permission.clone(), write_permission);
    println!("Read+Write permission: {:?}", read_write);
    
    // Admin OR Owner
    let admin_or_owner = guards::Guard::or(
        admin_permission,
        guards::Guard::owner_only()
    );
    println!("Admin or Owner: {:?}\n", admin_or_owner);
}

fn demo_usage_guards() {
    println!("=== Usage-based Guards ===");
    
    // Single use
    let single_use = guards::Guard::max_uses(1);
    println!("Single-use guard: {:?}", single_use);
    
    // Limited uses (e.g., trial period)
    let trial_access = guards::Guard::max_uses(10);
    println!("Trial access (10 uses): {:?}", trial_access);
    
    // Combine with time: Limited uses within time window
    let limited_trial = guards::Guard::and(
        guards::Guard::max_uses(100),
        guards::Guard::expires_at(1234567890 + 86400 * 30) // 30 days
    );
    println!("Limited trial (100 uses in 30 days): {:?}\n", limited_trial);
}

// Example session parameters for different use cases
fn example_session_params() {
    println!("=== Example Session Parameters ===");
    
    // Public read-only session
    let public_readonly = CreateSessionParams {
        guard: guards::Guard::allow_all(),
        shared_data: SessionSharedData::default(),
    };
    println!("Public read-only session: {:?}", public_readonly);
    
    // Owner-only with expiration
    let owner_temp = CreateSessionParams {
        guard: guards::Guard::and(
            guards::Guard::owner_only(),
            guards::Guard::expires_at(1234567890 + 3600)
        ),
        shared_data: SessionSharedData::default(),
    };
    println!("Owner temporary session: {:?}", owner_temp);
    
    // Multi-signature with usage limit
    let multisig_limited = CreateSessionParams {
        guard: guards::Guard::and(
            guards::Guard::multisig_pattern(Pubkey::new_unique()),
            guards::Guard::max_uses(5)
        ),
        shared_data: SessionSharedData::default(),
    };
    println!("Multi-sig limited session: {:?}", multisig_limited);
}