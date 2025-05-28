use anchor_lang::solana_program::pubkey::Pubkey;

/// Helper function to check if a program ID matches the spl_memo ID
pub fn is_memo_program(program_id: &Pubkey) -> bool {
    // Use the known memo program IDs directly
    let memo_program_id = spl_memo::ID;
    let memo_v1_program_id = spl_memo::v1::ID;
    
    program_id == &memo_program_id || program_id == &memo_v1_program_id
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_memo_program() {
        assert!(is_memo_program(&spl_memo::ID));
        assert!(is_memo_program(&spl_memo::v1::ID));
        
        let random_pubkey = Pubkey::new_unique();
        assert!(!is_memo_program(&random_pubkey));
    }
} 