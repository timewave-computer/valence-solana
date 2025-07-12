//! Test that SDK types serialize/deserialize correctly

#[cfg(test)]
mod tests {
    use valence_sdk::*;
    use solana_sdk::pubkey::Pubkey;
    
    #[test]
    fn test_account_request_serde() {
        let request = AccountRequest {
            id: Pubkey::new_unique(),
            owner: Pubkey::new_unique(),
            capabilities: vec!["transfer".to_string(), "mint".to_string()],
            init_state_hash: [1u8; 32],
            created_at: 12345,
        };
        
        // Serialize to JSON
        let json = serde_json::to_string(&request).unwrap();
        
        // Deserialize back
        let deserialized: AccountRequest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(request.id, deserialized.id);
        assert_eq!(request.owner, deserialized.owner);
        assert_eq!(request.capabilities, deserialized.capabilities);
        assert_eq!(request.init_state_hash, deserialized.init_state_hash);
        assert_eq!(request.created_at, deserialized.created_at);
    }
    
    #[test]
    fn test_session_serde() {
        let session = Session {
            id: Pubkey::new_unique(),
            owner: Pubkey::new_unique(),
            accounts: vec![Pubkey::new_unique(), Pubkey::new_unique()],
            namespace: "test-namespace".to_string(),
            is_consumed: false,
            nonce: 42,
            created_at: 12345,
            metadata: vec![1, 2, 3, 4],
        };
        
        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();
        
        assert_eq!(session.id, deserialized.id);
        assert_eq!(session.accounts.len(), deserialized.accounts.len());
        assert_eq!(session.namespace, deserialized.namespace);
    }
    
    #[test]
    fn test_operation_with_optional_diff() {
        let op1 = Operation {
            function_hash: [0u8; 32],
            args: vec![1, 2, 3],
            expected_diff: None,
            target_account: None,
        };
        
        let op2 = Operation {
            function_hash: [1u8; 32],
            args: vec![4, 5, 6],
            expected_diff: Some([2u8; 32]),
            target_account: Some(Pubkey::new_unique()),
        };
        
        // Test both serialize correctly
        let json1 = serde_json::to_string(&op1).unwrap();
        let json2 = serde_json::to_string(&op2).unwrap();
        
        let d1: Operation = serde_json::from_str(&json1).unwrap();
        let d2: Operation = serde_json::from_str(&json2).unwrap();
        
        assert!(d1.expected_diff.is_none());
        assert!(d2.expected_diff.is_some());
    }
}