use anchor_lang::prelude::*;
mod id;
use id::ID;

use clockwork_sdk::{
    state::{Thread, ThreadAccount, ThreadResponse},
};

#[program]
pub mod hello_clockwork {
    use super::*;

    pub fn hello_ix(_ctx: Context<HelloClockwork>) -> Result<ThreadResponse> {
        msg!("Hello! The current time is: {}", Clock::get().unwrap().unix_timestamp);
        // For Cron Triggered Thread, returning ::default() is enough
        // More on this in another guide
        Ok(ThreadResponse::default())
    }
}

#[derive(Accounts)]
pub struct HelloClockwork {}

// Add this to enforce that the ix can only be run from a given thread
// #[derive(Accounts)]
// pub struct HelloClockwork<'info> {
//     #[account(address = thread.pubkey(), signer)]
//     pub thread: Account<'info, Thread>,
// }
