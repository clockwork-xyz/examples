use {
    crate::state::*,
    anchor_lang::prelude::*,
    clockwork_sdk::{queue_program::accounts::Queue, CrankResponse},
};

#[derive(Accounts)]
pub struct DisbursePayment<'info> {
    #[account(
        mut,
        address = Subscriber::pubkey(subscriber.owner.key(),subscription.key()),
    )]
    pub subscriber: Account<'info, Subscriber>,

    #[account(
        mut,
        address = Subscription::pubkey(subscription.owner,subscription.subscription_id.clone())
    )]
    pub subscription: Box<Account<'info, Subscription>>,
    #[account(address = Queue::pubkey(subscription.key(), "subscription".into()))]
    pub subscriptions_queue: Box<Account<'info, Queue>>,
}

impl<'info> DisbursePayment<'_> {
    pub fn process(&mut self) -> Result<CrankResponse> {
        let Self {
            subscriber,
            subscription,
            ..
        } = self;

        let amount_left = subscriber
            .locked_amount
            .checked_sub(subscription.recurrent_amount);

        match amount_left {
            Some(value) => {
                subscriber.locked_amount = value;
                subscriber.is_subscribed = true;
            }
            None => {
                subscriber.is_subscribed = false;
            }
        }

        Ok(CrankResponse {
            next_instruction: None,
            kickoff_instruction: None,
        })
    }
}
