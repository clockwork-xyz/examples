use {
    crate::state::*,
    anchor_lang::prelude::*,
    anchor_lang::solana_program::instruction::Instruction,
    clockwork_sdk::queue_program::{
        self,
        accounts::{Queue, Trigger},
        QueueProgram,
    },
};

#[derive(Accounts)]
pub struct CreateQueue<'info> {
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
    pub system_program: Program<'info, System>,
}

impl<'info> CreateQueue<'_> {
    pub fn process(&mut self, bump: u8) -> Result<()> {
        let Self {
            payer,
            subscriber,
            clockwork_program,
            subscription,
            subscriptions_queue,
            system_program,
            ..
        } = self;

        let disburse_payment_ix = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(subscriber.key(), false),
                AccountMeta::new_readonly(subscription.key(), false),
                AccountMeta::new_readonly(subscriptions_queue.key(), true),
            ],
            data: clockwork_sdk::anchor_sighash("disburse_payment").into(),
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
            "payment".into(),
            disburse_payment_ix.into(),
            Trigger::Cron {
                schedule: subscription.schedule.clone(),
                skippable: false,
            },
        )?;

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

        Ok(())
    }
}
