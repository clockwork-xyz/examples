use {
    crate::state::*,
    anchor_lang::prelude::*,
    anchor_spl::token::{self, Token, TokenAccount, Transfer},
    clockwork_sdk::{
        thread_program::{
            self,
            accounts::{Thread, ThreadAccount},
            ThreadProgram,
        },
        ThreadResponse,
    },
};

#[derive(Accounts)]
pub struct DisbursePayment<'info> {
    #[account(
        mut,
        address = Subscriber::pda(subscriber.owner.key(),subscription.key()).0,
    )]
    pub subscriber: Account<'info, Subscriber>,
    #[account(mut)]
    pub subscriber_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = Subscription::pda(subscription.owner,subscription.subscription_id.clone()).0
    )]
    pub subscription: Account<'info, Subscription>,
    #[account(
        signer,
        address = subscription_thread.pubkey(),
        constraint = subscription_thread.authority.eq(&subscriber.key()),
        constraint = subscription_thread.id.eq("subscriber_thread"),
    )]
    pub subscription_thread: Box<Account<'info, Thread>>,
    #[account(
        mut,
        token::mint = subscription.mint,
        token::authority = subscription,
        address = Subscription::bank_pda(subscription.key(),subscription.owner.key()).0
    )]
    pub subscription_bank: Account<'info, TokenAccount>,

    #[account(address = thread_program::ID)]
    pub thread_program: Program<'info, ThreadProgram>,
    pub token_program: Program<'info, Token>,
}

impl<'info> DisbursePayment<'_> {
    pub fn process(&mut self) -> Result<ThreadResponse> {
        let Self {
            subscriber,
            subscription,
            subscription_thread,
            thread_program,
            subscriber_token_account,
            token_program,
            subscription_bank,
            ..
        } = self;

        if !subscriber.is_active || !subscription.is_active {
            clockwork_sdk::thread_program::cpi::thread_stop(CpiContext::new_with_signer(
                thread_program.to_account_info(),
                clockwork_sdk::thread_program::cpi::accounts::ThreadStop {
                    authority: subscription.to_account_info(),
                    thread: subscription_thread.to_account_info(),
                },
                &[&[
                    SEED_SUBSCRIBER,
                    subscriber.owner.as_ref(),
                    subscription.key().as_ref(),
                    &[subscriber.bump],
                ]],
            ))?;
        } else {
            let amount_left = subscriber_token_account
                .amount
                .checked_sub(subscription.recurrent_amount);
            match amount_left {
                Some(_) => {
                    subscriber.is_active = true;
                    subscriber.last_transfer_at = Some(Clock::get().unwrap().unix_timestamp);
                    token::transfer(
                        CpiContext::new_with_signer(
                            token_program.to_account_info(),
                            Transfer {
                                authority: subscriber.to_account_info(),
                                from: subscriber_token_account.to_account_info(),
                                to: subscription_bank.to_account_info(),
                            },
                            &[&[
                                SEED_SUBSCRIBER,
                                subscriber.owner.as_ref(),
                                subscription.key().as_ref(),
                                &[subscriber.bump],
                            ]],
                        ),
                        subscription.recurrent_amount,
                    )?;
                }
                None => {
                    subscriber.is_active = false;
                    clockwork_sdk::thread_program::cpi::thread_stop(CpiContext::new_with_signer(
                        thread_program.to_account_info(),
                        clockwork_sdk::thread_program::cpi::accounts::ThreadStop {
                            authority: subscriber.to_account_info(),
                            thread: subscription_thread.to_account_info(),
                        },
                        &[&[
                            SEED_SUBSCRIBER,
                            subscriber.owner.as_ref(),
                            subscription.key().as_ref(),
                            &[subscriber.bump],
                        ]],
                    ))?;
                }
            }
        }

        Ok(ThreadResponse::default())
    }
}
