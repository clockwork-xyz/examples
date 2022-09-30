use {
    crate::state::*,
    anchor_lang::prelude::*,
    clockwork_sdk::queue_program::{self, state::{CrankResponse, SEED_QUEUE, Queue}},
};

#[derive(Accounts)]
pub struct HelloWorld<'info> {
    #[account(seeds = [SEED_AUTHORITY], bump)]
    pub authority: Account<'info, Authority>,

    #[account(
        signer, 
        seeds = [
            SEED_QUEUE, 
            authority.key().as_ref(), 
            "hello".as_bytes()
        ], 
        seeds::program = queue_program::ID,
        bump,
        has_one = authority
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
