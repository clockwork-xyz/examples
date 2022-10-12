use {
    anchor_lang::prelude::*,
    clockwork_sdk::{
        queue_program::accounts::{Queue, QueueAccount},
        CrankResponse,
    },
};
#[derive(Accounts)]
#[instruction(name: String)]
pub struct HelloWorld<'info> {
    #[account(address = hello_queue.pubkey(), signer)]
    pub hello_queue: Account<'info, Queue>,
}

pub fn handler(_ctx: Context<HelloWorld>, name: String) -> Result<CrankResponse> {
    msg!(
        "Hello {}! The current time is: {}",
        name,
        Clock::get().unwrap().unix_timestamp
    );

    Ok(CrankResponse {
        next_instruction: None,
        kickoff_instruction: None,
    })
}
