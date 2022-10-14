use {
    crate::state::*,
    anchor_lang::prelude::*,
    anchor_lang::solana_program::instruction::Instruction,
    anchor_spl::token::{transfer, Token, TokenAccount, Transfer},
    clockwork_sdk::queue_program::{
        self,
        accounts::{Queue, Trigger},
        QueueProgram,
    },
    std::mem::size_of,
};

#[derive(Accounts)]
pub struct Subscribe<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        address = Subscriber::pubkey(payer.key(),subscription.key()),
        payer = payer,
        space = 8 + size_of::<Subscriber>(),
    )]
    pub subscriber: Account<'info, Subscriber>,
    #[account(
        mut,
        associated_token::authority = payer,
        associated_token::mint = subscription.mint,
    )]
    pub subscriber_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        address=Subscription::bank_pubkey(subscription.key(),subscription.owner.key())
    )]
    pub subscription_bank: Box<Account<'info, TokenAccount>>,

    #[account(address = queue_program::ID)]
    pub clockwork_program: Program<'info, QueueProgram>,
    #[account(address = Queue::pubkey(subscription.key(), "subscription".into()))]
    pub subscriptions_queue: Box<Account<'info, Queue>>,

    #[account(mut, address = Subscription::pubkey(subscription.owner.key(),subscription.subscription_id))]
    pub subscription: Account<'info, Subscription>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> Subscribe<'_> {
    pub fn process(&mut self, bump: u8, subscription_period: u8) -> Result<()> {
        let Self {
            payer,
            subscriber,
            subscriber_token_account,
            subscription_bank,
            clockwork_program,
            subscription,
            subscriptions_queue,
            system_program,
            token_program,
            ..
        } = self;

        subscriber.new(
            payer.key(),
            subscription.key(),
            (subscription_period as u64 - 1) * subscription.recurrent_amount,
            true,
        )?;

        transfer(
            CpiContext::new(
                token_program.to_account_info(),
                Transfer {
                    from: subscriber_token_account.to_account_info(),
                    to: subscription_bank.to_account_info(),
                    authority: payer.to_account_info(),
                },
            ),
            subscription_period as u64 * subscription.recurrent_amount,
        )?;

        let disburse_payment_ix = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(subscriber.key(), false),
                AccountMeta::new_readonly(subscription.key(), false),
                AccountMeta::new_readonly(subscriptions_queue.key(), true),
            ],
            data: clockwork_sdk::queue_program::utils::anchor_sighash("disburse_payment").into(),
        };

        clockwork_sdk::queue_program::cpi::queue_create(
            CpiContext::new_with_signer(
                clockwork_program.to_account_info(),
                clockwork_sdk::queue_program::cpi::accounts::QueueCreate {
                    authority: subscription.to_account_info(),
                    payer: payer.to_account_info(),
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
            disburse_payment_ix.into(),
            "payment".into(),
            Trigger::Cron {
                schedule: subscription.schedule,
                skippable: true,
            },
        )?;

        Ok(())
    }
}
