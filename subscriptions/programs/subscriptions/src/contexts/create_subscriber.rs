use {
    crate::state::*,
    anchor_lang::prelude::*,
    anchor_lang::solana_program::instruction::Instruction,
    anchor_spl::token::{self, Approve, Token},
    anchor_spl::token::{Mint, TokenAccount},
    clockwork_sdk::thread_program::{
        self,
        accounts::{Thread, Trigger},
        ThreadProgram,
    },
    std::mem::size_of,
};

#[derive(Accounts)]
pub struct CreateSubscriber<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        seeds =[
            SEED_SUBSCRIBER, payer.key().as_ref(), subscription.key().as_ref()
        ],
        bump,
        payer = payer,
        space = 8 + size_of::<Subscriber>(),
    )]
    pub subscriber: Account<'info, Subscriber>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = payer
    )]
    pub subscriber_token_account: Account<'info, TokenAccount>,
    #[account(address = Subscription::pda(subscription.owner.key(),subscription.subscription_id.clone()).0)]
    pub subscription: Account<'info, Subscription>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = subscription,
        address = Subscription::bank_pda(subscription.key(),subscription.owner.key()).0
    )]
    pub subscription_bank: Account<'info, TokenAccount>,
    #[account(address = Thread::pubkey(subscriber.key(),"subscriber_thread".to_string()))]
    pub subscription_thread: SystemAccount<'info>,
    pub mint: Account<'info, Mint>,

    #[account(address = thread_program::ID)]
    pub thread_program: Program<'info, ThreadProgram>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> CreateSubscriber<'_> {
    pub fn process(&mut self, subscriber_bump: u8) -> Result<()> {
        let Self {
            payer,
            subscriber,
            subscription,
            subscription_thread,
            thread_program,
            system_program,
            subscriber_token_account,
            token_program,
            subscription_bank,
            ..
        } = self;

        subscriber.new(payer.key(), subscription.key(), false, subscriber_bump)?;

        let disburse_payment_ix = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(subscriber.key(), false),
                AccountMeta::new(subscriber_token_account.key(), false),
                AccountMeta::new(subscription.key(), false),
                AccountMeta::new_readonly(subscription_thread.key(), true),
                AccountMeta::new(subscription_bank.key(), false),
                AccountMeta::new_readonly(thread_program.key(), false),
                AccountMeta::new_readonly(token_program.key(), false),
            ],
            data: clockwork_sdk::anchor_sighash("disburse_payment").into(),
        };

        token::approve(
            CpiContext::new(
                token_program.to_account_info(),
                Approve {
                    authority: payer.to_account_info(),
                    delegate: subscriber.to_account_info(),
                    to: subscriber_token_account.to_account_info(),
                },
            ),
            u64::MAX,
        )?;

        clockwork_sdk::thread_program::cpi::thread_create(
            CpiContext::new_with_signer(
                thread_program.to_account_info(),
                clockwork_sdk::thread_program::cpi::accounts::ThreadCreate {
                    authority: subscriber.to_account_info(),
                    payer: payer.to_account_info(),
                    system_program: system_program.to_account_info(),
                    thread: subscription_thread.to_account_info(),
                },
                &[&[
                    SEED_SUBSCRIBER,
                    payer.key().as_ref(),
                    subscription.key().as_ref(),
                    &[subscriber_bump],
                ]],
            ),
            "subscriber_thread".to_string(),
            disburse_payment_ix.into(),
            Trigger::Cron {
                schedule: subscription.schedule.clone(),
                skippable: false,
            },
        )?;

        clockwork_sdk::thread_program::cpi::thread_pause(CpiContext::new_with_signer(
            thread_program.to_account_info(),
            clockwork_sdk::thread_program::cpi::accounts::ThreadPause {
                authority: subscriber.to_account_info(),
                thread: subscription_thread.to_account_info(),
            },
            &[&[
                SEED_SUBSCRIBER,
                payer.key().as_ref(),
                subscription.key().as_ref(),
                &[subscriber_bump],
            ]],
        ))?;

        Ok(())
    }
}
