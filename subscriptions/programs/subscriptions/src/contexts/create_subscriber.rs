use {
    crate::state::*,
    anchor_lang::prelude::*,
    anchor_lang::solana_program::instruction::Instruction,
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
    #[account(address = Subscription::pda(subscription.owner.key(),subscription.subscription_id.clone()).0)]
    pub subscription: Account<'info, Subscription>,
    #[account(address = Thread::pubkey(subscription.key(),subscription.subscription_id.to_string()))]
    pub subscription_thread: SystemAccount<'info>,

    #[account(address = thread_program::ID)]
    pub thread_program: Program<'info, ThreadProgram>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateSubscriber<'_> {
    pub fn process(&mut self) -> Result<()> {
        let Self {
            payer,
            subscriber,
            subscription,
            subscription_thread,
            thread_program,
            system_program,
        } = self;

        subscriber.new(payer.key(), subscription.key(), 0, false, false)?;

        let disburse_payment_ix = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(subscriber.key(), false),
                AccountMeta::new(subscription.key(), false),
                AccountMeta::new_readonly(subscription_thread.key(), true),
                AccountMeta::new_readonly(thread_program::ID, false),
            ],
            data: clockwork_sdk::anchor_sighash("disburse_payment").into(),
        };

        clockwork_sdk::thread_program::cpi::thread_create(
            CpiContext::new_with_signer(
                thread_program.to_account_info(),
                clockwork_sdk::thread_program::cpi::accounts::ThreadCreate {
                    authority: subscription.to_account_info(),
                    payer: payer.to_account_info(),
                    system_program: system_program.to_account_info(),
                    thread: subscription_thread.to_account_info(),
                },
                &[&[
                    SEED_SUBSCRIPTION,
                    subscription.owner.as_ref(),
                    &subscription.subscription_id.to_be_bytes(),
                    &[subscription.bump],
                ]],
            ),
            subscription.subscription_id.to_string(),
            disburse_payment_ix.into(),
            Trigger::Account {
                address: subscriber.key(),
                offset: 0,
                size: 8,
            },
        )?;

        Ok(())
    }
}
