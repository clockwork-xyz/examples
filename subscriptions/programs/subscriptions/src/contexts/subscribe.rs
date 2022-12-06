use {
    crate::{error::ErrorCode, state::*},
    anchor_lang::prelude::*,
    anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer},
    clockwork_sdk::thread_program::{self, accounts::Thread, ThreadProgram},
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
        mut,
        associated_token::mint = mint,
        associated_token::authority = payer
    )]
    pub subscriber_token_account: Account<'info, TokenAccount>,

    #[account(mut, address = Subscription::pda(subscription.owner.key(),subscription.subscription_id.clone()).0)]
    pub subscription: Account<'info, Subscription>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = subscription,
        address = Subscription::bank_pda(subscription.key(),subscription.owner.key()).0
    )]
    pub subscription_bank: Account<'info, TokenAccount>,
    #[account(address = Thread::pubkey(subscriber.key(),"subscriber_thread".to_string()))]
    pub subscription_thread: Box<Account<'info, Thread>>,

    #[account(address=subscription.mint)]
    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    #[account(address = thread_program::ID)]
    pub thread_program: Program<'info, ThreadProgram>,
}

impl<'info> Subscribe<'_> {
    pub fn process(&mut self) -> Result<()> {
        let Self {
            payer,
            subscriber,
            subscription,
            subscriber_token_account,
            token_program,
            subscription_bank,
            thread_program,
            subscription_thread,
            ..
        } = self;

        require!(
            subscriber_token_account.amount >= subscription.recurrent_amount,
            ErrorCode::InsuffiscientAmount
        );
        require!(subscription.is_active, ErrorCode::SubscriptionInactive);

        token::transfer(
            CpiContext::new(
                token_program.to_account_info(),
                Transfer {
                    authority: payer.to_account_info(),
                    from: subscriber_token_account.to_account_info(),
                    to: subscription_bank.to_account_info(),
                },
            ),
            subscription.recurrent_amount,
        )?;

        clockwork_sdk::thread_program::cpi::thread_resume(CpiContext::new_with_signer(
            thread_program.to_account_info(),
            clockwork_sdk::thread_program::cpi::accounts::ThreadResume {
                authority: subscriber.to_account_info(),
                thread: subscription_thread.to_account_info(),
            },
            &[&[
                SEED_SUBSCRIBER,
                payer.key().as_ref(),
                subscription.key().as_ref(),
                &[subscriber.bump],
            ]],
        ))?;

        subscriber.is_active = true;
        subscriber.last_transfer_at = Some(Clock::get().unwrap().unix_timestamp);
        Ok(())
    }
}
