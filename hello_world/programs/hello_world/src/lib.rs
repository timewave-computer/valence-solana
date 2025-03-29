use anchor_lang::prelude::*;

declare_id!("ERdxmzFtjsA29ceTC6MF9qtfDA1qLV2S3krStCCbbJbF");

#[program]
pub mod hello_world {
    use super::*;

    // Initialize the program with a greeting account
    pub fn initialize(ctx: Context<Initialize>, greeting: String) -> Result<()> {
        let greeting_account = &mut ctx.accounts.greeting_account;
        greeting_account.greeting = greeting;
        greeting_account.counter = 0;
        msg!("Greeting account initialized with: {}", greeting_account.greeting);
        Ok(())
    }

    // Update the greeting
    pub fn update_greeting(ctx: Context<UpdateGreeting>, new_greeting: String) -> Result<()> {
        let greeting_account = &mut ctx.accounts.greeting_account;
        greeting_account.greeting = new_greeting;
        greeting_account.counter += 1;
        msg!("Greeting updated to: {}", greeting_account.greeting);
        msg!("Counter increased to: {}", greeting_account.counter);
        Ok(())
    }

    // Say hello and increment counter
    pub fn say_hello(ctx: Context<SayHello>) -> Result<()> {
        let greeting_account = &mut ctx.accounts.greeting_account;
        greeting_account.counter += 1;
        msg!("Hello, {}!", greeting_account.greeting);
        msg!("You've said hello {} times", greeting_account.counter);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 32 + 8 // 8 bytes for discriminator, 32 for string, 8 for counter
    )]
    pub greeting_account: Account<'info, GreetingAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateGreeting<'info> {
    #[account(mut)]
    pub greeting_account: Account<'info, GreetingAccount>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct SayHello<'info> {
    #[account(mut)]
    pub greeting_account: Account<'info, GreetingAccount>,
}

#[account]
pub struct GreetingAccount {
    pub greeting: String,
    pub counter: u64,
} 