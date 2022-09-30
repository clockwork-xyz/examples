use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{
            instruction::Instruction, system_program, sysvar,
        },
    },
    anchor_spl::{
        associated_token::{self, AssociatedToken},
        token::{Mint, TokenAccount},
    },
    clockwork_sdk::queue_program::{self, QueueProgram, state::{Trigger, SEED_QUEUE}},
    std::mem::size_of,
};

#[derive(Accounts)]
#[instruction(disbursement_amount: u64, schedule: String)]
pub struct CreatePayment<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(address = queue_program::ID)]
    pub clockwork_program: Program<'info, QueueProgram>,

    #[account(
        init,
        payer = sender,
        associated_token::authority = payment,
        associated_token::mint = mint,
    )]
    pub escrow: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = sender,
        seeds = [
            SEED_PAYMENT, 
            sender.key().as_ref(), 
            recipient.key().as_ref(), 
            mint.key().as_ref()
        ],
        bump,
        space = 8 + size_of::<Payment>(),
    )]
    pub payment: Account<'info, Payment>,

    #[account(
        seeds = [
            SEED_QUEUE, 
            payment.key().as_ref(), 
            "payment".as_bytes()
        ], 
        seeds::program = queue_program::ID,
        bump
    )]
    pub payment_queue: SystemAccount<'info>,

    /// CHECK: the recipient is validated by the seeds of the payment account
    #[account()]
    pub recipient: AccountInfo<'info>,

    #[account(
        associated_token::authority = recipient,
        associated_token::mint = mint,
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(mut)]
    pub sender: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, CreatePayment<'info>>,
    disbursement_amount: u64,
    schedule: String,
) -> Result<()> {
    // Get accounts
    let clockwork_program = &ctx.accounts.clockwork_program;
    let escrow = &ctx.accounts.escrow;
    let mint = &ctx.accounts.mint;
    let payment = &mut ctx.accounts.payment;
    let payment_queue = &mut ctx.accounts.payment_queue;
    let recipient = &ctx.accounts.recipient;
    let recipient_token_account = &ctx.accounts.recipient_token_account;
    let sender = &ctx.accounts.sender;
    let system_program = &ctx.accounts.system_program;
    let token_program = &ctx.accounts.token_program;

    // get payment bump
    let bump = *ctx.bumps.get("payment").unwrap();

    // initialize payment account
    payment.new(
        sender.key(),
        recipient.key(),
        mint.key(),
        0,
        disbursement_amount,
        schedule,
    )?;

    // create ix
    let disburse_payment_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new(escrow.key(), false),
            AccountMeta::new_readonly(payment.mint, false),
            AccountMeta::new(payment.key(), false),
            AccountMeta::new_readonly(payment_queue.key(), true),
            AccountMeta::new_readonly(payment.recipient, false),
            AccountMeta::new(recipient_token_account.key(), false),
            AccountMeta::new_readonly(payment.sender, false),
            AccountMeta::new_readonly(token_program.key(), false),
        ],
        data: clockwork_sdk::queue_program::utils::anchor_sighash("disburse_payment").into(),
    };

    msg!("payment: {:#?}", payment);

    // Create queue
    clockwork_sdk::queue_program::cpi::queue_create(
        CpiContext::new_with_signer(
            clockwork_program.to_account_info(),
            clockwork_sdk::queue_program::cpi::accounts::QueueCreate {
                authority: payment.to_account_info(),
                payer: sender.to_account_info(),
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
        "payment".into(),
        disburse_payment_ix.into(),
        Trigger::Cron {
            schedule: payment.schedule.to_string()
        },
    )?;

    Ok(())
}
