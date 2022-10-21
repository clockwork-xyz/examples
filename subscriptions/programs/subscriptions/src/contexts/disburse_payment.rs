use {
    crate::state::*,
    anchor_lang::prelude::*,
    clockwork_sdk::{
        queue_program::{self, accounts::Queue, QueueProgram},
        CrankResponse,
    },
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
    pub subscription: Account<'info, Subscription>,
    #[account(address = Queue::pubkey(subscription.key(), "subscription".into()))]
    pub subscriptions_queue: Account<'info, Queue>,

    #[account(address = queue_program::ID)]
    pub clockwork_program: Program<'info, QueueProgram>,
}

impl<'info> DisbursePayment<'_> {
    pub fn process(&mut self, bump: u8) -> Result<CrankResponse> {
        let Self {
            subscriber,
            subscription,
            subscriptions_queue,
            clockwork_program,
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
                clockwork_sdk::queue_program::cpi::queue_stop(CpiContext::new_with_signer(
                    clockwork_program.to_account_info(),
                    clockwork_sdk::queue_program::cpi::accounts::QueueStop {
                        authority: subscription.to_account_info(),
                        queue: subscriptions_queue.to_account_info(),
                    },
                    &[&[
                        SEED_SUBSCRIPTION,
                        subscription.owner.as_ref(),
                        subscription.subscription_id.as_bytes(),
                        &[bump],
                    ]],
                ))?;
            }
        }

        Ok(CrankResponse {
            next_instruction: None,
            kickoff_instruction: None,
        })
    }
}
