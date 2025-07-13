use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct EchoInput {
    pub message: String,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct EchoOutput {
    pub response: String,
}

/// Echo function that returns the input message with "echo: " prefix
pub fn process_echo(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    input_data: &[u8],
) -> ProgramResult {
    msg!("Echo function called");
    
    // Deserialize input
    let input = EchoInput::try_from_slice(input_data)?;
    msg!("Received message: {}", input.message);
    
    // Create output
    let output = EchoOutput {
        response: format!("echo: {}", input.message),
    };
    
    // For now, just log the output
    msg!("Returning: {}", output.response);
    
    Ok(())
}
