use anchor_lang::prelude::*;
mod id;
use id::ID;

#[program]
pub mod hello_clockwork {
    use super::*;

    pub fn hello_ix(_ctx: Context<HelloClockwork>) -> Result<()> {
        msg!("Hello! The current time is: {}", Clock::get().unwrap().unix_timestamp);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct HelloClockwork {}
