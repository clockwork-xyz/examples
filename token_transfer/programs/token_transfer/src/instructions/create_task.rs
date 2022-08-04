use {
    crate::state::*,
    anchor_lang::{ 
        prelude::*,
        solana_program::{instruction::Instruction, system_program},
    },
    anchor_spl::{token::TokenAccount, associated_token::{self, AssociatedToken}},
    clockwork_scheduler::state::{SEED_TASK, SEED_QUEUE, Queue}
};

#[derive(Accounts)]
pub struct CreateTask<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(
        seeds = [SEED_AUTHORITY], 
        bump
    )]
    pub authority: Account<'info, Authority>,

    #[account(
        seeds = [SEED_ESCROW, sender.key().as_ref(), recipient.key().as_ref()],
        bump,
        has_one = recipient
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
      seeds = [SEED_QUEUE, authority.key().as_ref(), "token_transfer_queue".as_ref()], 
      seeds::program = clockwork_scheduler::ID, 
      bump,
    )]
    pub queue: Account<'info, Queue>,

    #[account()]
    pub recipient: AccountInfo<'info>,

    #[account(
        associated_token::authority = escrow.recipient,
        associated_token::mint = escrow.mint,
    )]
    pub recipient_token_account: Box<Account<'info, TokenAccount>>,

    #[account(address = clockwork_scheduler::ID)]
    pub scheduler_program: Program<'info, clockwork_scheduler::program::ClockworkScheduler>,

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

    #[account(
        associated_token::authority = escrow,
        associated_token::mint = escrow.mint,
    )]
    pub vault: Box<Account<'info, TokenAccount>>,
}

pub fn handler<'info> (
  ctx: Context<'_, '_, '_, 'info, CreateTask<'info>>
) -> Result<()> {
    // Get Accounts
    let authority = &ctx.accounts.authority;
    let escrow = &ctx.accounts.escrow;
    let queue = &ctx.accounts.queue;
    let recipient_token_account = &ctx.accounts.recipient_token_account;
    let scheduler_program = &ctx.accounts.scheduler_program;
    let sender = &ctx.accounts.sender;
    let system_program = &ctx.accounts.system_program;
    let task = &ctx.accounts.task;
    let token_program = &ctx.accounts.token_program;
    let vault = &ctx.accounts.vault;    

    // get authority bump
    let bump = *ctx.bumps.get("authority").unwrap();
    
    // create disburse ix
    let disburse_payment_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new_readonly(authority.key(), false),
            AccountMeta::new_readonly(escrow.key(), false),
            AccountMeta::new_readonly(escrow.mint.key(), false),
            AccountMeta::new_readonly(queue.key(), true),
            AccountMeta::new_readonly(escrow.recipient.key(), false),
            AccountMeta::new(recipient_token_account.key(), false),
            AccountMeta::new_readonly(escrow.sender.key(), false),
            AccountMeta::new(vault.key(), false),
            AccountMeta::new_readonly(token_program.key(), false),
        ],
        data: clockwork_scheduler::anchor::sighash("disburse_payment").into(),
    };

    // Create task with the disburse ix and add it to the queue
    clockwork_scheduler::cpi::task_new(
        CpiContext::new_with_signer(
            scheduler_program.to_account_info(),
            clockwork_scheduler::cpi::accounts::TaskNew {
                authority: authority.to_account_info(),
                payer: sender.to_account_info(),
                queue: queue.to_account_info(),
                system_program: system_program.to_account_info(),
                task: task.to_account_info(),
            },
            &[&[SEED_AUTHORITY, &[bump]]],
        ),
        vec![disburse_payment_ix.into()],
    )?;

  Ok(())
}