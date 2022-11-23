use {
    crate::state::*,
    anchor_lang::prelude::*,
    clockwork_sdk::{
        ExecResponse,
        thread_program::{
            self,
            ThreadProgram, 
            accounts::{
                ThreadAccount, Thread, 
            }},
    },
};

#[derive(Accounts)]
pub struct DisbursePayment<'info> {
    #[account(
        mut,
        address = Subscriber::pda(subscriber.owner.key(),subscription.key()).0,
    )]
    pub subscriber: Account<'info, Subscriber>,

    #[account(
        mut,
        address = Subscription::pda(subscription.owner,subscription.subscription_id.clone()).0
    )]
    pub subscription: Account<'info, Subscription>,
    #[account(
        signer, 
        address = thread.pubkey(),
        constraint = thread.authority.eq(&subscription.owner),
    )]
    pub thread: Box<Account<'info, Thread>>,

    #[account(address = thread_program::ID)]
    pub clockwork_program: Program<'info, ThreadProgram>,
}

impl<'info> DisbursePayment<'_> {
    pub fn process(&mut self) -> Result<ExecResponse> {
        let Self {
            subscriber,
            subscription,
            thread,
            clockwork_program,
            ..
        } = self;

        if !subscriber.is_active || !subscription.is_active {
            subscriber.is_subscribed = false;
            clockwork_sdk::thread_program::cpi::thread_stop(CpiContext::new_with_signer(
                clockwork_program.to_account_info(),
                clockwork_sdk::thread_program::cpi::accounts::ThreadStop {
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
        } else {
            let amount_left = subscriber
                .locked_amount
                .checked_sub(subscription.recurrent_amount);

            match amount_left {
                Some(value) => {
                    subscriber.locked_amount = value;
                    subscriber.is_subscribed = true;
                    subscription.withdraw += subscription.recurrent_amount;
                }
                None => {
                    subscriber.is_subscribed = false;
                    clockwork_sdk::thread_program::cpi::thread_stop(CpiContext::new_with_signer(
                        clockwork_program.to_account_info(),
                        clockwork_sdk::thread_program::cpi::accounts::ThreadStop {
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
                }
            }
        }

        Ok(ExecResponse::default())
    }
}
