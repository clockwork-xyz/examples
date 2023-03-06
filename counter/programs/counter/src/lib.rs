use anchor_lang::prelude::*;
use anchor_lang::{solana_program::{system_program}};
pub mod id;
pub use id::ID;

pub const SEED_COUNTER: &[u8] = b"counter";

#[program]
pub mod counter {
    use super::*;

    pub fn increment(ctx: Context<Increment>) -> Result<clockwork_sdk::state::ThreadResponse> {
        ctx.accounts.counter.current_value = ctx.accounts.counter.current_value.checked_add(1).unwrap();
        ctx.accounts.counter.updated_at = Clock::get().unwrap().unix_timestamp;

        msg!(
            "Counter value: {}, updated_at: {}",
            ctx.accounts.counter.current_value,
            ctx.accounts.counter.updated_at
        );

        Ok(clockwork_sdk::state::ThreadResponse::default())
    }

    // pub fn increment_thread() -> Result<()> {
        
    // }
}

#[derive(Accounts)]
pub struct Increment<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        seeds = [SEED_COUNTER],
        bump,
        space = 8 + std::mem::size_of::<Counter>(),
    )]
    pub counter: Account<'info, Counter>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(Debug)]
pub struct Counter {
    pub current_value: u64,
    pub updated_at: i64,
}
