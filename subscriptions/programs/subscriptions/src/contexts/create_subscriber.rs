use {crate::state::*, anchor_lang::prelude::*, std::mem::size_of};

#[derive(Accounts)]
pub struct CreateSubscriber<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        seeds =[
            SEED_SUBSCRIBER, payer.key().as_ref(), subscription.key().as_ref()
        ],
        bump,
        payer = payer,
        space = 8 + size_of::<Subscriber>(),
    )]
    pub subscriber: Account<'info, Subscriber>,

    #[account(address = Subscription::pubkey(subscription.owner.key(),subscription.subscription_id.clone()))]
    pub subscription: Account<'info, Subscription>,

    pub system_program: Program<'info, System>,
}

impl<'info> CreateSubscriber<'_> {
    pub fn process(&mut self) -> Result<()> {
        let Self {
            payer,
            subscriber,
            subscription,
            ..
        } = self;

        subscriber.new(payer.key(), subscription.key(), 0, false, false)?;

        Ok(())
    }
}
