use {
    crate::state::*,
    clockwork_sdk::queue_program::state::Queue,
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct ProcessEvent<'info> {
    #[account(seeds = [SEED_AUTHORITY], bump)]
    pub authority: Account<'info, Authority>,

    #[account(seeds = [SEED_EVENT], bump)]
    pub event: Account<'info, Event>,

    #[account(
        seeds = [
            clockwork_sdk::queue_program::state::SEED_QUEUE, 
            authority.key().as_ref(), 
            "events".as_bytes()
        ], 
        seeds::program = clockwork_sdk::queue_program::ID,
        bump,
        signer,
        has_one = authority
    )]
    pub queue: Account<'info, Queue>,
}

pub fn handler(ctx: Context<ProcessEvent>) -> Result<()> {
    // Get accounts
    let event = &mut ctx.accounts.event;

    // Initialize event account
    msg!("Event was triggered by {} at {}", event.user, event.timestamp);

    Ok(())
}
