use solana_program::pubkey::Pubkey;

/// Helper function to convert spl_memo's Pubkey to a solana_program's Pubkey
pub fn convert_pubkey(pubkey: &spl_memo::solana_program::solana_pubkey::Pubkey) -> Pubkey {
    Pubkey::new_from_array(pubkey.to_bytes())
}

/// Helper function to check if a program ID matches the spl_memo ID
pub fn is_memo_program(program_id: &Pubkey) -> bool {
    // Convert the spl_memo::id to a solana_program::pubkey::Pubkey
    let memo_pubkey = convert_pubkey(&spl_memo::id());
    let memo_v1_pubkey = convert_pubkey(&spl_memo::v1::id());
    
    program_id == &memo_pubkey || program_id == &memo_v1_pubkey
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_convert_pubkey() {
        let memo_pubkey = spl_memo::id();
        let converted = convert_pubkey(&memo_pubkey);
        assert_eq!(memo_pubkey.to_bytes(), converted.to_bytes());
    }
    
    #[test]
    fn test_is_memo_program() {
        let memo_pubkey = convert_pubkey(&spl_memo::id());
        assert!(is_memo_program(&memo_pubkey));
        
        let memo_v1_pubkey = convert_pubkey(&spl_memo::v1::id());
        assert!(is_memo_program(&memo_v1_pubkey));
        
        let random_pubkey = Pubkey::new_unique();
        assert!(!is_memo_program(&random_pubkey));
    }
} 