use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod hello_world_function {
    use super::*;

    /// Process function - called by the shard through CPI
    pub fn process(ctx: Context<Process>, input_data: Vec<u8>) -> Result<()> {
        msg!("Hello World function called with {} bytes", input_data.len());
        
        // Deserialize input
        let input = HelloWorldInput::try_from_slice(&input_data)?;
        msg!("Received name: {}", input.name);
        
        // Get current timestamp
        let clock = Clock::get()?;
        
        // Create greeting
        let greeting = if input.name.is_empty() {
            "Hello, World!".to_string()
        } else {
            format!("Hello, {}!", input.name)
        };
        
        // Create output
        let output = HelloWorldOutput {
            greeting: greeting.clone(),
            timestamp: clock.unix_timestamp,
        };
        
        msg!("Greeting: {}", output.greeting);
        msg!("Timestamp: {}", output.timestamp);
        
        // In a real implementation, you might write output to an account
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Process<'info> {
    /// The shard that called this function
    /// CHECK: Validated by the shard program
    pub shard: AccountInfo<'info>,
    
    /// The user who initiated the function call
    pub user: Signer<'info>,
}

/// Input structure for the hello world function
#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct HelloWorldInput {
    pub name: String,
}

/// Output structure for the hello world function
#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct HelloWorldOutput {
    pub greeting: String,
    pub timestamp: i64,
}

/// Function metadata that would be used for registration
pub const HELLO_WORLD_METADATA: FunctionMetadata = FunctionMetadata {
    name: "hello_world",
    version: "1.0.0",
    description: "A simple hello world function",
    capabilities_required: 0, // No special capabilities needed
};

#[derive(Debug)]
pub struct FunctionMetadata {
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    pub capabilities_required: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world_serialization() {
        // Test input serialization
        let input = HelloWorldInput {
            name: "Alice".to_string(),
        };
        let serialized = borsh::to_vec(&input).unwrap();
        let deserialized = HelloWorldInput::try_from_slice(&serialized).unwrap();
        assert_eq!(input.name, deserialized.name);

        // Test output serialization
        let output = HelloWorldOutput {
            greeting: "Hello, Alice!".to_string(),
            timestamp: 1234567890,
        };
        let serialized = borsh::to_vec(&output).unwrap();
        let deserialized = HelloWorldOutput::try_from_slice(&serialized).unwrap();
        assert_eq!(output.greeting, deserialized.greeting);
        assert_eq!(output.timestamp, deserialized.timestamp);
    }
}