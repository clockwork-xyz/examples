use {
    crate::state::*,
    anchor_lang::prelude::*,
    clockwork_sdk::state::Thread,
};

#[derive(Accounts)]
pub struct ProcessEvent<'info> {
    #[account(address = Authority::pubkey())]
    pub authority: Account<'info, Authority>,

    #[account(address = Event::pubkey())]
    pub event: Account<'info, Event>,

    #[account(
        address = event_thread.key(),
        signer,
        has_one = authority
    )]
    pub event_thread: Account<'info, Thread>,
}

pub fn handler(ctx: Context<ProcessEvent>) -> Result<()> {
    // Get accounts
    let event = &mut ctx.accounts.event;

    // Initialize event account
    msg!(
        "Event was triggered by {} at {}",
        event.user,
        event.timestamp,
    );

    Ok(())
}
