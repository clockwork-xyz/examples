use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::sysvar},
    cronos_scheduler::state::Manager,
};

#[derive(Accounts)]
pub struct HelloWorld<'info> {
    #[account(seeds = [SEED_AUTHORITY], bump, has_one = manager)]
    pub authority: Box<Account<'info, Authority>>,

    #[account(signer, constraint = manager.authority == authority.key())]
    pub manager: Box<Account<'info, Manager>>,

    #[account(address = sysvar::clock::ID)]
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<HelloWorld>) -> Result<()> {
    let clock = &ctx.accounts.clock;

    msg!(
        "Hello world! the unix timestamp is: {}",
        clock.unix_timestamp
    );

    Ok(())
}
