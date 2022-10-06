use {
    crate::state::*,
    anchor_lang::prelude::*,
    clockwork_crank::{program::ClockworkCrank, state::SEED_QUEUE},
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
        mint: Pubkey,
        is_active: bool,
    ) -> Result<()> {
        let Self {
            payer,
            subscription,
            ..
        } = self;

        subscription.new(payer.key(), mint, recurrent_amount, epochs_reset, is_active)?;

        Ok(())
    }
}
