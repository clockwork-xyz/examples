use {
    crate::{error::ErrorCode, state::*},
    anchor_lang::prelude::*,
    clockwork_sdk::queue_program::{self, accounts::Queue, QueueProgram},
};

#[derive(Accounts)]
pub struct Subscribe<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        address = Subscriber::pubkey(payer.key(),subscription.key()),
    )]
    pub subscriber: Account<'info, Subscriber>,

    #[account(address = Queue::pubkey(subscription.key(), "subscription".into()))]
    pub subscriptions_queue: Box<Account<'info, Queue>>,
    #[account(mut, address = Subscription::pubkey(subscription.owner.key(),subscription.subscription_id.clone()))]
    pub subscription: Account<'info, Subscription>,

    #[account(address = queue_program::ID)]
    pub clockwork_program: Program<'info, QueueProgram>,
}

impl<'info> Subscribe<'_> {
    pub fn process(&mut self, bump: u8) -> Result<()> {
        let Self {
            subscriber,
            clockwork_program,
            subscription,
            subscriptions_queue,
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

        clockwork_sdk::queue_program::cpi::queue_resume(CpiContext::new_with_signer(
            clockwork_program.to_account_info(),
            clockwork_sdk::queue_program::cpi::accounts::QueueResume {
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

        Ok(())
    }
}
