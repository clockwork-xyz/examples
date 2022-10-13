use {
    crate::state::*,
    anchor_lang::prelude::*,
    clockwork_crank::state::{CrankResponse, Queue, SEED_QUEUE},
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
    #[account(
        signer,
        seeds = [
            SEED_QUEUE,
            subscription.key().as_ref(),
            "subscription".as_bytes()
        ],
        seeds::program = clockwork_crank::ID,
        bump,
    )]
    pub subscription_queue: Box<Account<'info, Queue>>,
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
                subscriber.is_active = false;
                subscriber.is_subscribed = false;
            }
            None => {
                subscriber.locked_amount -= subscription.recurrent_amount;
                subscriber.is_subscribed = true
            }
        }

        Ok(CrankResponse {
            next_instruction: None,
        })
    }
}
