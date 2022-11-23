use {
    crate::{error::ErrorCode, state::*},
    anchor_lang::prelude::*,
    clockwork_sdk::thread_program::{
            self,
            accounts::{Thread, ThreadAccount},
            ThreadProgram,
        },
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

    #[account(
        signer, 
        address = thread.pubkey(),
        constraint = thread.authority.eq(&subscription.owner),
    )]
    pub thread: Box<Account<'info, Thread>>,
    #[account(mut, address = Subscription::pda(subscription.owner.key(),subscription.subscription_id.clone()).0)]
    pub subscription: Account<'info, Subscription>,

    #[account(address = thread_program::ID)]
    pub clockwork_program: Program<'info, ThreadProgram>,
}

impl<'info> Subscribe<'_> {
    pub fn process(&mut self) -> Result<()> {
        let Self {
            subscriber,
            clockwork_program,
            subscription,
            thread,
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

        clockwork_sdk::thread_program::cpi::thread_resume(CpiContext::new_with_signer(
            clockwork_program.to_account_info(),
            clockwork_sdk::thread_program::cpi::accounts::ThreadResume {
                authority: subscription.to_account_info(),
                thread: thread.to_account_info(),
            },
            &[&[
                SEED_SUBSCRIPTION,
                subscription.owner.as_ref(),
                &subscription.subscription_id.to_be_bytes(),
                &[subscription.bump],
            ]],
        ))?;

        Ok(())
    }
}
