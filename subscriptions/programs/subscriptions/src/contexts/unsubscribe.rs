use clockwork_sdk::queue_program::accounts::QueueSettings;

use {
    crate::state::*,
    anchor_lang::prelude::*,
    anchor_spl::token::{transfer, Token, TokenAccount, Transfer},
    clockwork_sdk::queue_program::{self, accounts::Queue, QueueProgram},
};

#[derive(Accounts)]
pub struct Unsubscribe<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        address = Subscriber::pubkey(payer.key(),subscription.key()),
    )]
    pub subscriber: Account<'info, Subscriber>,
    #[account(
        mut,
        associated_token::authority = payer,
        associated_token::mint = subscription.mint,
    )]
    pub subscriber_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        address=Subscription::bank_pubkey(subscription.key(),subscription.owner.key())
    )]
    pub subscription_bank: Account<'info, TokenAccount>,

    #[account(mut, address = Subscription::pubkey(subscription.owner.key(),subscription.subscription_id.clone()))]
    pub subscription: Account<'info, Subscription>,
    #[account(address = Queue::pubkey(subscription.key(), "subscription".into()))]
    pub subscriptions_queue: Box<Account<'info, Queue>>,

    #[account(address = queue_program::ID)]
    pub clockwork_program: Program<'info, QueueProgram>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> Unsubscribe<'_> {
    pub fn process(&mut self, bump: u8) -> Result<()> {
        let Self {
            subscriber,
            subscriber_token_account,
            subscription_bank,
            subscription,
            system_program,
            token_program,
            clockwork_program,
            subscriptions_queue,
            ..
        } = self;

        transfer(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                Transfer {
                    from: subscription_bank.to_account_info(),
                    to: subscriber_token_account.to_account_info(),
                    authority: subscription.to_account_info(),
                },
                &[&[
                    subscription.key().as_ref(),
                    subscription.owner.as_ref(),
                    "subscription_bank".as_bytes(),
                ]],
            ),
            subscriber.locked_amount,
        )?;

        subscriber.locked_amount = 0;

        queue_program::cpi::queue_update(
            CpiContext::new_with_signer(
                clockwork_program.to_account_info(),
                clockwork_sdk::queue_program::cpi::accounts::QueueUpdate {
                    authority: subscription.to_account_info(),
                    queue: subscriptions_queue.to_account_info(),
                    system_program: system_program.to_account_info(),
                },
                &[&[
                    SEED_SUBSCRIPTION,
                    subscription.owner.as_ref(),
                    subscription.subscription_id.as_bytes(),
                    &[bump],
                ]],
            ),
            // Which values to use here ?
            QueueSettings {
                fee: None,
                kickoff_instruction: None,
                rate_limit: None,
                trigger: None,
            },
        )?;

        Ok(())
    }
}
