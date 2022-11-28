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
        ExecResponse,
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
        address = thread.pubkey(),
        constraint = thread.authority.eq(&subscription.owner),
        constraint = thread.id.eq("subscription"),
    )]
    pub thread: Box<Account<'info, Thread>>,
    #[account(
        mut,
        token::mint = subscription.mint,
        token::authority = subscription,
        address = Subscription::bank_pda(subscription.key(),subscription.owner.key()).0
    )]
    pub subscription_bank: Account<'info, TokenAccount>,

    #[account(address = thread_program::ID)]
    pub clockwork_program: Program<'info, ThreadProgram>,
    pub token_program: Program<'info, Token>,
}

impl<'info> DisbursePayment<'_> {
    pub fn process(&mut self) -> Result<ExecResponse> {
        let Self {
            subscriber,
            subscription,
            thread,
            clockwork_program,
            subscriber_token_account,
            token_program,
            subscription_bank,
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
            let amount_left = subscriber_token_account
                .amount
                .checked_sub(subscription.recurrent_amount);
            match amount_left {
                Some(_) => {
                    subscriber.is_subscribed = true;
                    token::transfer(
                        CpiContext::new_with_signer(
                            token_program.to_account_info(),
                            Transfer {
                                authority: subscription.to_account_info(),
                                from: subscriber_token_account.to_account_info(),
                                to: subscription_bank.to_account_info(),
                            },
                            &[&[
                                SEED_SUBSCRIPTION,
                                subscription.owner.as_ref(),
                                &subscription.subscription_id.to_be_bytes(),
                                &[subscription.bump],
                            ]],
                        ),
                        subscription.recurrent_amount,
                    )?;
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
