use {crate::state::*, anchor_lang::prelude::*};

#[derive(Accounts)]
pub struct Ping<'info> {
    #[account(mut, seeds = [SEED_EVENT], bump)]
    pub event: Account<'info, Event>,

    #[account(mut)]
    pub signer: Signer<'info>,
}

pub fn handler(ctx: Context<Ping>) -> Result<()> {
    // Get accounts
    let event = &mut ctx.accounts.event;
    let signer = &ctx.accounts.signer;

    // Initialize event account
    event.user = signer.key();
    event.timestamp = Clock::get().unwrap().unix_timestamp;

    Ok(())
}
