
use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{instruction::Instruction, system_program, sysvar},
    },
    anchor_spl::token::{self, Mint, TokenAccount},
    clockwork_scheduler::state::{Queue, SEED_QUEUE, SEED_TASK}
};

#[derive(Accounts)]
pub struct CreateTask<'info> {
    #[account(
        seeds = [SEED_AUTHORITY, authority.payer.as_ref()], 
        bump
    )]
    pub authority: Account<'info, Authority>,

    #[account(address = anchor_spl::dex::ID)]
    pub dex_program: Program<'info, anchor_spl::dex::Dex>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account()]
    pub pc_mint: Account<'info, Mint>,

    #[account(mut, token::authority = authority, token::mint = pc_mint)]
    pub pc_wallet: Account<'info, TokenAccount>,

    #[account(
      seeds = [SEED_QUEUE, authority.key().as_ref(), "dca_queue".as_ref()], 
      seeds::program = clockwork_scheduler::ID, 
      bump,
    )]
    pub queue: Account<'info, Queue>,

    #[account(address = clockwork_scheduler::ID)]
    pub scheduler_program: Program<'info, clockwork_scheduler::program::ClockworkScheduler>,

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

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, CreateTask<'info>>) -> Result<()> {
    // Get Accounts
    let authority = &ctx.accounts.authority;
    let dex_program = &ctx.accounts.dex_program;
    let payer = &ctx.accounts.payer;
    let pc_mint = &ctx.accounts.pc_mint;
    let pc_wallet = &mut ctx.accounts.pc_wallet;
    let queue = &ctx.accounts.queue;
    let scheduler_program = &ctx.accounts.scheduler_program;
    let system_program = &ctx.accounts.system_program;
    let task = &ctx.accounts.task;

    // Get remaining Accounts
    let market = ctx.remaining_accounts.get(0).unwrap();
    let coin_vault = ctx.remaining_accounts.get(1).unwrap();
    let pc_vault = ctx.remaining_accounts.get(2).unwrap();
    let request_queue = ctx.remaining_accounts.get(3).unwrap();
    let event_queue = ctx.remaining_accounts.get(4).unwrap();
    let market_bids = ctx.remaining_accounts.get(5).unwrap();
    let market_asks = ctx.remaining_accounts.get(6).unwrap();
    let open_orders = ctx.remaining_accounts.get(7).unwrap();

    // get authority bump
    let bump = *ctx.bumps.get("authority").unwrap();

    // create swap ix
    let swap_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority.key(), false),
            AccountMeta::new_readonly(dex_program.key(), false),
            AccountMeta::new(clockwork_scheduler::payer::ID, true),
            AccountMeta::new_readonly(pc_mint.key(), false),
            AccountMeta::new(pc_wallet.key(), false),
            AccountMeta::new(queue.key(), true),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
            // Extra Accounts
            AccountMeta::new(market.key(), false),
            AccountMeta::new(coin_vault.key(), false),
            AccountMeta::new(pc_vault.key(), false),
            AccountMeta::new(request_queue.key(), false),
            AccountMeta::new(event_queue.key(), false),
            AccountMeta::new(market_bids.key(), false),
            AccountMeta::new(market_asks.key(), false),
            AccountMeta::new(open_orders.key(), false),
        ],
        data: clockwork_scheduler::anchor::sighash("swap").into(),
    };

    // Create task with the disburse ix and add it to the queue
    clockwork_scheduler::cpi::task_new(
        CpiContext::new_with_signer(
            scheduler_program.to_account_info(),
            clockwork_scheduler::cpi::accounts::TaskNew {
                authority: authority.to_account_info(),
                payer: payer.to_account_info(),
                queue: queue.to_account_info(),
                system_program: system_program.to_account_info(),
                task: task.to_account_info(),
            },
            &[&[SEED_AUTHORITY, authority.payer.as_ref(), &[bump]]],
        ),
        vec![swap_ix.into()],
    )?;

    Ok(())
}
