use {
    crate::state::*,
    anchor_lang::prelude::*,
    clockwork_sdk::queue_program::accounts::{CrankResponse, Queue, QueueAccount},
};

#[derive(Accounts)]
pub struct HelloWorld<'info> {
    #[account(address = Authority::pubkey())]
    pub authority: Account<'info, Authority>,

    #[account(
        address = hello_queue.pubkey(),
        constraint = hello_queue.id.eq("hello"),
        has_one = authority,
        signer, 
    )]
    pub hello_queue: Account<'info, Queue>,
}

pub fn handler(_ctx: Context<HelloWorld>) -> Result<CrankResponse> {
    msg!(
        "Hello world! The current time is: {}",
        Clock::get().unwrap().unix_timestamp
    );

    Ok(CrankResponse {
        next_instruction: None,
    })
}
