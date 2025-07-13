use solana_program::{
    account_info::AccountInfo,
    declare_id,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

declare_id!("11111111111111111111111111111111");

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Echo program entrypoint");
    
    if instruction_data.is_empty() {
        msg!("No instruction data provided");
        return Ok(());
    }
    
    match instruction_data[0] {
        0 => {
            msg!("Initialize instruction");
            Ok(())
        }
        1 => {
            msg!("Echo instruction");
            if instruction_data.len() > 1 {
                // Just log the number of bytes received
                msg!("Received {} bytes", instruction_data.len() - 1);
            }
            Ok(())
        }
        _ => {
            msg!("Unknown instruction");
            Ok(())
        }
    }
}
