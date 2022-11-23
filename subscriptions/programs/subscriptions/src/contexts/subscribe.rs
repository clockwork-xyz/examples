use {
    crate::{error::ErrorCode, state::*},
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct Subscribe<'info> {
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

impl<'info> Subscribe<'_> {
    pub fn process(&mut self) -> Result<()> {
        let Self {
            subscriber,
            subscription,
            ..
        } = self;

        require!(
            subscriber.locked_amount >= subscription.recurrent_amount,
            ErrorCode::InsuffiscientAmountLocked
        );
        require!(subscription.is_active, ErrorCode::SubscriptionInactive);

        subscriber.is_active = true;
        subscriber.is_subscribed = true;
        subscriber.locked_amount -= subscription.recurrent_amount;
        subscription.withdraw += subscription.recurrent_amount;
        Ok(())
    }
}
