use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{
            native_token::LAMPORTS_PER_SOL, system_program,sysvar, instruction::Instruction
        },
    },
    anchor_spl::{associated_token::{self, AssociatedToken}, token::{Mint, TokenAccount}},
    clockwork_scheduler::{state::{SEED_QUEUE, SEED_TASK}, program::ClockworkScheduler},
    std::mem::size_of,
};

#[derive(Accounts)]
#[instruction(disbursement_amount: u64, schedule: String)]
pub struct CreatePayment<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

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
        seeds = [SEED_PAYMENT, sender.key().as_ref(), recipient.key().as_ref(), mint.key().as_ref()],
        bump,
        space = 8 + size_of::<Payment>(),
    )]
    pub payment: Account<'info, Payment>, 

    #[account(
        seeds = [SEED_QUEUE, payment.key().as_ref(), "payment_queue".as_bytes()], 
        seeds::program = clockwork_scheduler::ID, 
        bump,
	)]
    pub queue: SystemAccount<'info>,

    #[account()]
    pub recipient: AccountInfo<'info>,

    #[account(
        associated_token::authority = recipient,
        associated_token::mint = mint,
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = clockwork_scheduler::ID)]
    pub scheduler_program: Program<'info, ClockworkScheduler>,

    #[account(mut)]
    pub sender: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(
        seeds = [SEED_TASK, queue.key().as_ref(), (0 as u64).to_be_bytes().as_ref()], 
        seeds::program = clockwork_scheduler::ID, 
        bump
	)]
	pub task: SystemAccount<'info>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,

}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, CreatePayment<'info>>, disbursement_amount: u64, schedule: String) -> Result<()> {
    // Get accounts
    let escrow = &ctx.accounts.escrow;
    let mint = &ctx.accounts.mint;
    let payment = &mut ctx.accounts.payment;
    let queue = &mut ctx.accounts.queue;
    let recipient = &ctx.accounts.recipient;
    let recipient_token_account = &ctx.accounts.recipient_token_account;
    let scheduler_program = &ctx.accounts.scheduler_program;
    let sender = &ctx.accounts.sender;
    let system_program = &ctx.accounts.system_program;
    let task = &mut ctx.accounts.task;
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
       schedule
    )?;

    // Create queue
    clockwork_scheduler::cpi::queue_new(
        CpiContext::new_with_signer(
            scheduler_program.to_account_info(),
            clockwork_scheduler::cpi::accounts::QueueNew {
                authority: payment.to_account_info(),
                payer: sender.to_account_info(),
                queue: queue.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[&[SEED_PAYMENT, payment.sender.as_ref(), payment.recipient.as_ref(), payment.mint.as_ref(), &[bump]]]
        ),
        LAMPORTS_PER_SOL,
        "payment_queue".to_string(),
        payment.schedule.to_string(),
    )?;

     // create ix
    let disburse_payment_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new(escrow.key(), false),
            AccountMeta::new_readonly(payment.mint, false),
            AccountMeta::new(payment.key(), false),
            AccountMeta::new_readonly(queue.key(), true),
            AccountMeta::new_readonly(payment.recipient, false),
            AccountMeta::new(recipient_token_account.key(), false),
            AccountMeta::new_readonly(payment.sender, false),
            AccountMeta::new_readonly(token_program.key(), false),
        ],
        data: clockwork_scheduler::anchor::sighash("disburse_payment").into(),
    };

    // Create task with ix
    clockwork_scheduler::cpi::task_new(
        CpiContext::new_with_signer(
            scheduler_program.to_account_info(),
            clockwork_scheduler::cpi::accounts::TaskNew {
                authority: payment.to_account_info(),
                payer: sender.to_account_info(),
                queue: queue.to_account_info(),
                system_program: system_program.to_account_info(),
                task: task.to_account_info(),
            },
            &[&[SEED_PAYMENT, payment.sender.as_ref(), payment.recipient.as_ref(), payment.mint.as_ref(), &[bump]]]
        ),
        vec![disburse_payment_ix.into()],
    )?;


    Ok(())
}
