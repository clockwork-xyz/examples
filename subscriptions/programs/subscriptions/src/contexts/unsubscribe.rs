use {crate::state::*, anchor_lang::prelude::*};

#[derive(Accounts)]
pub struct Unsubscribe<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        address = Subscriber::pda(payer.key(),subscription.key()).0,
    )]
    pub subscriber: Account<'info, Subscriber>,

    #[account(mut, address = Subscription::pda(subscription.owner.key(),subscription.subscription_id.clone()).0)]
    pub subscription: Account<'info, Subscription>,
}

impl<'info> Unsubscribe<'_> {
    pub fn process(&mut self) -> Result<()> {
        let Self { subscriber, .. } = self;

        subscriber.is_active = false;

        Ok(())
    }
}
