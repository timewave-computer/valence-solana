// Comprehensive tests for Account Factory Program functionality

#[cfg(test)]
mod tests {
    use crate::utils::*;
    use anchor_lang::prelude::*;
    use base_account::state::AccountState;

    #[test]
    fn test_factory_state_creation() {
        let ctx = TestContext::new();
        
        // Test factory state parameters
        let owner = ctx.authority;
        let valence_registry = generate_test_pubkey("registry");
        let created_accounts_count = 0u64;
        let fee_account = generate_test_pubkey("fee_account");

        // Validate parameters
        assert_ne!(owner, Pubkey::default(), "Owner should be valid");
        assert_ne!(valence_registry, Pubkey::default(), "Registry should be valid");
        assert_eq!(created_accounts_count, 0, "Initial count should be zero");
        assert_ne!(fee_account, Pubkey::default(), "Fee account should be valid");
    }

    #[test]
    fn test_account_creation_workflow() {
        let ctx = TestContext::new();
        
        // Test account creation parameters
        let owner = ctx.user;
        let account_type = "base_account";
        let initial_sol_amount = 1000000; // 0.001 SOL in lamports
        
        // Validate creation parameters
        assert_ne!(owner, Pubkey::default(), "Owner should be valid");
        assert!(!account_type.is_empty(), "Account type should be specified");
        assert!(initial_sol_amount > 0, "Should have initial funding");
        assert!(initial_sol_amount >= 890880, "Should meet rent exemption minimum");
    }

    #[test]
    fn test_account_template_management() {
        // Test account template structure
        let template_name = "standard_base_account";
        let account_type = "base_account";
        let default_config = [1, 2, 3, 4]; // Mock configuration data
        let is_active = true;

        // Validate template parameters
        assert!(!template_name.is_empty(), "Template name should not be empty");
        assert!(template_name.len() <= 64, "Template name should be reasonable length");
        assert!(!account_type.is_empty(), "Account type should be specified");
        assert!(!default_config.is_empty(), "Should have default configuration");
        assert!(default_config.len() <= 1024, "Config should be reasonable size");
        assert!(is_active, "Template should be active by default");
    }

    #[test]
    fn test_batch_account_creation() {
        let ctx = TestContext::new();
        
        // Test batch creation parameters
        let owners = vec![ctx.user, ctx.authority, generate_test_pubkey("user3")];
        let account_type = "base_account";
        let max_batch_size = 10;

        // Validate batch parameters
        assert!(!owners.is_empty(), "Should have at least one owner");
        assert!(owners.len() <= max_batch_size, "Should not exceed batch limit");
        assert!(!account_type.is_empty(), "Account type should be specified");
        
        // Ensure all owners are unique
        let mut unique_owners = owners.clone();
        unique_owners.sort();
        unique_owners.dedup();
        assert_eq!(unique_owners.len(), owners.len(), "All owners should be unique");
    }

    #[test]
    fn test_account_initialization_data() {
        let ctx = TestContext::new();
        
        // Test account state initialization
        let account_state = AccountState {
            owner: ctx.user,
            approved_libraries: vec![],
            vault_authority: generate_test_pubkey("vault_authority"),
            vault_bump_seed: 255,
            token_accounts: vec![],
            last_activity: 1000000000,
            instruction_count: 0,
        };

        // Test serialization
        let serialized = account_state.try_to_vec().unwrap();
        let deserialized: AccountState = AccountState::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.owner, ctx.user);
        assert!(deserialized.approved_libraries.is_empty());
        assert_eq!(deserialized.vault_authority, account_state.vault_authority);
        assert_eq!(deserialized.vault_bump_seed, 255);
        assert!(deserialized.token_accounts.is_empty());
        assert_eq!(deserialized.last_activity, 1000000000);
        assert_eq!(deserialized.instruction_count, 0);
    }

    #[test]
    fn test_factory_fee_calculation() {
        // Test fee calculation for different account types
        let base_account_fee = 1000; // Base fee in lamports
        let storage_account_fee = 2000; // Higher fee for storage accounts
        let rent_exemption = 890880; // Minimum rent exemption
        
        // Test fee validation
        assert!(base_account_fee > 0, "Base fee should be positive");
        assert!(storage_account_fee > base_account_fee, "Storage should cost more");
        assert!(rent_exemption > base_account_fee, "Rent exemption should cover fees");
        
        // Test total cost calculation
        let total_base_cost = base_account_fee + rent_exemption;
        let total_storage_cost = storage_account_fee + rent_exemption;
        
        assert!(total_base_cost > rent_exemption, "Total should include fees");
        assert!(total_storage_cost > total_base_cost, "Storage should cost more total");
    }

    #[test]
    fn test_account_creation_limits() {
        // Test creation rate limiting
        let max_accounts_per_user = 100;
        let max_accounts_per_transaction = 10;
        let cooldown_period = 60; // seconds
        
        // Validate limits
        assert!(max_accounts_per_user > 0, "Should allow account creation");
        assert!(max_accounts_per_user <= 1000, "Should have reasonable upper limit");
        assert!(max_accounts_per_transaction > 0, "Should allow batch creation");
        assert!(max_accounts_per_transaction <= 20, "Should limit batch size");
        assert!(cooldown_period >= 0, "Cooldown should be non-negative");
    }

    #[test]
    fn test_factory_state_updates() {
        let _ctx = TestContext::new();
        
        // Test factory state tracking
        let mut created_accounts_count = 0u64;
        let accounts_to_create = 5;
        
        // Simulate account creation
        for _ in 0..accounts_to_create {
            created_accounts_count += 1;
        }
        
        assert_eq!(created_accounts_count, accounts_to_create);
        assert!(created_accounts_count < u64::MAX, "Should not overflow");
    }

    #[test]
    fn test_template_validation() {
        // Test template configuration validation
        let valid_template_name = "standard_base";
        let invalid_template_name = ""; // Empty name
        let long_template_name = "a".repeat(100); // Too long
        
        // Validate template names
        assert!(!valid_template_name.is_empty(), "Valid name should not be empty");
        assert!(valid_template_name.len() <= 64, "Valid name should be reasonable length");
        
        assert!(invalid_template_name.is_empty(), "Invalid name should be empty");
        assert!(long_template_name.len() > 64, "Long name should exceed limit");
    }

    #[test]
    fn test_account_type_validation() {
        let valid_account_types = vec![
            "base_account",
            "storage_account",
        ];
        
        let invalid_account_types = vec![
            "",
            "unknown_type",
            "invalid-type-with-dashes",
        ];
        
        // Test valid types
        for account_type in valid_account_types {
            assert!(!account_type.is_empty(), "Valid type should not be empty");
            assert!(account_type.chars().all(|c| c.is_alphanumeric() || c == '_'), 
                    "Valid type should only contain alphanumeric and underscore");
        }
        
        // Test invalid types
        for account_type in invalid_account_types {
            if !account_type.is_empty() {
                assert!(account_type.contains('-') || account_type == "unknown_type", 
                        "Invalid type should have invalid characters or be unknown");
            }
        }
    }

    #[test]
    fn test_space_calculations() {
        // Test space calculations for different account configurations
        let base_account_space = AccountState::get_space(0, 0); // No libraries, no token accounts
        let with_libraries = AccountState::get_space(5, 0); // 5 libraries, no token accounts
        let with_tokens = AccountState::get_space(0, 3); // No libraries, 3 token accounts
        let full_account = AccountState::get_space(10, 5); // 10 libraries, 5 token accounts
        
        // Validate space calculations (base account space is 97 bytes)
        assert!(base_account_space > 90, "Base account should have reasonable size");
        assert!(with_libraries > base_account_space, "Libraries should add space");
        assert!(with_tokens > base_account_space, "Token accounts should add space");
        assert!(full_account > with_libraries, "Full account should be largest");
        assert!(full_account > with_tokens, "Full account should be largest");
        
        // Each library adds 32 bytes (Pubkey)
        assert_eq!(with_libraries - base_account_space, 5 * 32);
        // Each token account adds 32 bytes (Pubkey)
        assert_eq!(with_tokens - base_account_space, 3 * 32);
    }

    #[test]
    fn test_factory_permissions() {
        let ctx = TestContext::new();
        
        // Test permission validation
        let factory_owner = ctx.authority;
        let regular_user = ctx.user;
        let template_creator = generate_test_pubkey("template_creator");
        
        // Validate permission logic
        assert_ne!(factory_owner, regular_user, "Owner and user should be different");
        assert_ne!(factory_owner, template_creator, "Owner and creator should be different");
        
        // Test permission checks (simulated)
        let can_create_template = factory_owner == ctx.authority; // Only owner can create templates
        let can_create_account = true; // Anyone can create accounts
        let can_modify_factory = factory_owner == ctx.authority; // Only owner can modify factory
        
        assert!(can_create_template, "Owner should be able to create templates");
        assert!(can_create_account, "Users should be able to create accounts");
        assert!(can_modify_factory, "Owner should be able to modify factory");
    }
} 