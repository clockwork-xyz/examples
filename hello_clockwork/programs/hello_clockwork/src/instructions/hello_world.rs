use {
    crate::state::*,
    anchor_lang::prelude::*,
    clockwork_scheduler::{response::TaskResponse, state::Queue},
};

#[derive(Accounts)]
pub struct HelloWorld<'info> {
    #[account(seeds = [SEED_AUTHORITY], bump)]
    pub authority: Account<'info, Authority>,

    #[account(signer, has_one = authority)]
    pub queue: Account<'info, Queue>,
}

pub fn handler(_ctx: Context<HelloWorld>) -> Result<TaskResponse> {
    msg!(
        "Hello world! The current time is: {}",
        Clock::get().unwrap().unix_timestamp
    );

    Ok(TaskResponse::default())
}
