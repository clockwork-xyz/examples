use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::system_program},
    anchor_spl::token::Mint,
    clockwork_sdk::queue_program::{
        self,
        accounts::{Queue, QueueAccount, QueueSettings, Trigger},
        QueueProgram,
    },
};

#[derive(Accounts)]
#[instruction(amount: Option<u64>, trigger: Option<Trigger>)]
pub struct UpdatePayment<'info> {
    #[account(address = queue_program::ID)]
    pub clockwork_program: Program<'info, QueueProgram>,

    #[account()]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [
            SEED_PAYMENT,
            payment.sender.key().as_ref(),
            payment.recipient.key().as_ref(),
            payment.mint.key().as_ref()
        ],
        bump,
        has_one = recipient,
        has_one = sender,
        has_one = mint
    )]
    pub payment: Account<'info, Payment>,

    #[account(
        mut,
        address = payment_queue.pubkey(),
        constraint = payment_queue.authority.eq(&payment.sender),
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
    amount: Option<u64>,
    trigger: Option<Trigger>,
) -> Result<()> {
    // Get accounts
    let clockwork_program = &ctx.accounts.clockwork_program;
    let payment = &mut ctx.accounts.payment;
    let payment_queue = &mut ctx.accounts.payment_queue;
    let sender = &ctx.accounts.sender;
    let system_program = &ctx.accounts.system_program;

    // get payment bump
    let bump = *ctx.bumps.get("payment").unwrap();

    // update amount
    if let Some(amount) = amount {
        payment.amount = amount;
    }

    // update queue trigger
    if let Some(trigger) = trigger {
        clockwork_sdk::queue_program::cpi::queue_update(
            CpiContext::new_with_signer(
                clockwork_program.to_account_info(),
                clockwork_sdk::queue_program::cpi::accounts::QueueUpdate {
                    authority: sender.to_account_info(),
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
            QueueSettings {
                fee: None,
                kickoff_instruction: None,
                rate_limit: None,
                trigger: Some(trigger),
            },
        )?;
    }

    Ok(())
}
