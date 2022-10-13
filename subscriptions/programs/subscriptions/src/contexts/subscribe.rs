use {
    crate::state::*,
    anchor_lang::prelude::*,
    anchor_lang::solana_program::instruction::Instruction,
    anchor_spl::token::{transfer, Token, TokenAccount, Transfer},
    clockwork_crank::{
        program::ClockworkCrank,
        state::{Trigger, SEED_QUEUE},
    },
    std::mem::size_of,
};

#[derive(Accounts)]
pub struct Subscribe<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init_if_needed,
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
        associated_token::authority = subscription,
        associated_token::mint = subscription.mint,
    )]
    pub subscription_bank: Box<Account<'info, TokenAccount>>,

    #[account(address = clockwork_crank::ID)]
    pub clockwork_program: Program<'info, ClockworkCrank>,
    #[account(
        seeds = [
            SEED_QUEUE,
            subscription.key().as_ref(),
            "subscription".as_bytes()
        ],
        seeds::program = clockwork_crank::ID,
        bump
    )]
    pub subscriptions_queue: SystemAccount<'info>,

    #[account(mut)]
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
            subscription_period as u64 * subscription.recurrent_amount,
            true,
            false,
        )?;

        // transfer from subscriber to subscription_bank
        transfer(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                Transfer {
                    from: subscriber_token_account.to_account_info(),
                    to: subscription_bank.to_account_info(),
                    authority: subscription.to_account_info(),
                },
                &[&[
                    SEED_SUBSCRIPTION,
                    subscription.owner.as_ref(),
                    subscription.subscription_id.as_bytes(),
                    &[bump],
                ]],
            ),
            subscription_period as u64 * subscription.recurrent_amount,
        )?;

        // let disburse_payment_ix = Instruction {
        //     program_id: crate::ID,
        //     accounts: vec![
        //         AccountMeta::new_readonly(associated_token::ID, false),
        //         AccountMeta::new_readonly(subscription.mint, false),
        //         AccountMeta::new(subscription.key(), false),
        //         AccountMeta::new_readonly(subscriptions_queue.key(), true),
        //         AccountMeta::new_readonly(subscription.recipient, false),
        //         // AccountMeta::new(recipient_token_account.key(), false),
        //         AccountMeta::new_readonly(payer.key(), false),
        //         AccountMeta::new_readonly(token_program.key(), false),
        //     ],
        //     data: clockwork_crank::anchor::sighash("disburse_payment").into(),
        // };

        // clockwork_crank::cpi::queue_create(
        //     CpiContext::new_with_signer(
        //         clockwork_program.to_account_info(),
        //         clockwork_crank::cpi::accounts::QueueCreate {
        //             authority: subscription.to_account_info(),
        //             payer: payer.to_account_info(),
        //             queue: subscriptions_queue.to_account_info(),
        //             system_program: system_program.to_account_info(),
        //         },
        //         // FIX SEEDS
        //         &[&[SEED_SUBSCRIPTION, &[bump]]],
        //     ),
        //     disburse_payment_ix.into(),
        //     "payment".into(),
        //     // TIME SHOULD BE CURRENT + EPOCHS RESET
        //     Trigger::Cron {
        //         schedule: "12".to_string(),
        //     },
        // )?;

        Ok(())
    }
}
