use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::system_program},
    anchor_spl::token::Mint,
    clockwork_crank::{
        program::ClockworkCrank,
        state::{Queue, Trigger, SEED_QUEUE},
    },
};

#[derive(Accounts)]
#[instruction(disbursement_amount: Option<u64>, schedule: Option<Trigger>)]
pub struct UpdatePayment<'info> {
    #[account(address = clockwork_crank::ID)]
    pub clockwork_program: Program<'info, ClockworkCrank>,

    #[account()]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [SEED_PAYMENT, payment.sender.key().as_ref(), payment.recipient.key().as_ref(), payment.mint.as_ref()],
        bump,
        has_one = recipient,
        has_one = sender,
        has_one = mint
    )]
    pub payment: Account<'info, Payment>,

    #[account(
        mut,
        seeds = [
            SEED_QUEUE, 
            payment.key().as_ref(), 
            "payment".as_bytes()
        ],
        seeds::program = clockwork_crank::ID,
        bump,
	  )]
    pub payment_queue: Account<'info, Queue>,

    /// CHECK: this account is validated against the payment account
    #[account()]
    pub recipient: AccountInfo<'info>,

    #[account(mut)]
    pub sender: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, UpdatePayment<'info>>,
    disbursement_amount: Option<u64>,
    schedule: Option<Trigger>,
) -> Result<()> {
    // Get accounts
    let clockwork_program = &ctx.accounts.clockwork_program;
    let payment = &mut ctx.accounts.payment;
    let payment_queue = &mut ctx.accounts.payment_queue;
    let system_program = &ctx.accounts.system_program;

    // get payment bump
    let bump = *ctx.bumps.get("payment").unwrap();

    // update disbursement amount
    if let Some(disbursement_amount) = disbursement_amount {
        payment.disbursement_amount = disbursement_amount;
    }

    // update queue schedule
    if let Some(schedule) = schedule {
            // Update payment_queue schedule
            clockwork_crank::cpi::queue_update(
                CpiContext::new_with_signer(
                    clockwork_program.to_account_info(),
                    clockwork_crank::cpi::accounts::QueueUpdate {
                        authority: payment.to_account_info(),
                        queue: payment_queue.to_account_info(),
                        system_program: system_program.to_account_info(),
                    },
                    &[&[
                        SEED_PAYMENT,
                        payment.sender.as_ref(),
                        payment.recipient.as_ref(),
                        payment.mint.as_ref(),
                        &[bump],
                    ]],
                ),
                None,
                Some(schedule),
            )?;
    }

    Ok(())
}
