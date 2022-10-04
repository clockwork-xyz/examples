use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::system_program},
    clockwork_crank::{
        program::ClockworkCrank,
        state::{Trigger, SEED_QUEUE},
    },
    std::mem::size_of,
};

#[derive(Accounts)]
pub struct CreateSubscription<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(address = clockwork_crank::ID)]
    pub clockwork_program: Program<'info, ClockworkCrank>,
    #[account(
        seeds = [
            SEED_QUEUE,
            subscription.key().as_ref(),
            "subscription".as_bytes()
        ],
        seeds::program = clockwork_crank::ID,
        bump
    )]
    pub subscriptions_queue: SystemAccount<'info>,

    #[account(
        init,
        payer = payer,
        seeds = [
            SEED_SUBSCRIPTION,
            payer.key().as_ref(),
        ],
        bump,
        space = 8 + size_of::<Subscription>(),
    )]
    pub subscription: Account<'info, Subscription>,

    pub system_program: Program<'info, System>,
}

impl<'info> CreateSubscription<'_> {
    pub fn process(
        &mut self,
        recurrent_amount: u64,
        epochs_reset: u64,
        start_schedule: String,
    ) -> Result<()> {
        let Self {
            payer,
            clockwork_program,
            ..
        } = self;

        Ok(())
    }
}
