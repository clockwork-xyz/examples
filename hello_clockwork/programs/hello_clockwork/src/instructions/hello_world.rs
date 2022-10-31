use {
    anchor_lang::prelude::*,
    clockwork_sdk::{
        thread_program::accounts::{Thread, ThreadAccount},
        ExecResponse,
    },
};
#[derive(Accounts)]
#[instruction(name: String)]
pub struct HelloWorld<'info> {
    #[account(address = hello_thread.pubkey(), signer)]
    pub hello_thread: Account<'info, Thread>,
}

pub fn handler(_ctx: Context<HelloWorld>, name: String) -> Result<ExecResponse> {
    msg!(
        "Hello {}! The current time is: {}",
        name,
        Clock::get().unwrap().unix_timestamp
    );

    Ok(ExecResponse {
        next_instruction: None,
        kickoff_instruction: None,
    })
}
