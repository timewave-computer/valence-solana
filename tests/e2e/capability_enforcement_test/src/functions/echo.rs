//! Echo function implementation

use anchor_lang::prelude::*;

/// Echo function that logs the input message
pub fn process_echo(message: &str) -> Result<()> {
    msg!("Echo function called");
    msg!("Received message: {}", message);
    msg!("Returning: echo: {}", message);
    Ok(())
}