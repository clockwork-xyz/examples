use {
    crate::state::*,
    anchor_lang::prelude::*,
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

    #[account(
        init,
        payer = payer,
        space = 8 + size_of::<Subscription>(),
    )]
    pub subscription: Account<'info, Subscription>,

    pub system_program: Program<'info, System>,
}

impl<'info> Subscribe<'_> {
    pub fn process(&mut self, bump: u8) -> Result<()> {
        let Self {
            payer,
            clockwork_program,
            subscription,
            subscriptions_queue,
            system_program,
            ..
        } = self;

        // create ix
        // let disburse_payment_ix = Instruction {
        //     program_id: crate::ID,
        //     accounts: vec![
        //         AccountMeta::new_readonly(associated_token::ID, false),
        //         AccountMeta::new(escrow.key(), false),
        //         AccountMeta::new_readonly(payment.mint, false),
        //         AccountMeta::new(payment.key(), false),
        //         AccountMeta::new_readonly(payment_queue.key(), true),
        //         AccountMeta::new_readonly(payment.recipient, false),
        //         AccountMeta::new(recipient_token_account.key(), false),
        //         AccountMeta::new_readonly(payment.sender, false),
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
        //         &[&[
        //             SEED_SUBSCRIPTION,
        //             subscription.recipient.as_ref(),
        //             subscription.mint.as_ref(),
        //             &[bump],
        //         ]],
        //     ),
        //     disburse_payment_ix.into(),
        //     "payment".into(),
        //     Trigger::Cron { schedule: 12 },
        // )?;

        Ok(())
    }
}
