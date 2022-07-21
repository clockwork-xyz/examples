use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{instruction::Instruction, system_program, sysvar},
    },
    anchor_spl::token::{self, Mint, TokenAccount},
};

#[derive(Accounts)]
pub struct AutoSwap<'info> {
    #[account(seeds = [SEED_AUTHORITY, authority.payer.as_ref()], bump, has_one = manager)]
    pub authority: Account<'info, Authority>,

    #[account(address = sysvar::clock::ID)]
    pub clock: Sysvar<'info, Clock>,

    #[account(address = anchor_spl::dex::ID)]
    pub dex_program: Program<'info, anchor_spl::dex::Dex>,

    #[account(mut, has_one = authority)]
    pub manager: Account<'info, cronos_scheduler::state::Manager>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account()]
    pub pc_mint: Account<'info, Mint>,

    #[account(mut, token::authority = authority, token::mint = pc_mint)]
    pub pc_wallet: Account<'info, TokenAccount>,

    #[account(address = cronos_scheduler::ID)]
    pub scheduler_program: Program<'info, cronos_scheduler::program::CronosScheduler>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, AutoSwap<'info>>) -> Result<()> {
    // Get Accounts
    let authority = &ctx.accounts.authority;
    let clock = &ctx.accounts.clock;
    let dex_program = &ctx.accounts.dex_program;
    let manager = &mut ctx.accounts.manager;
    let payer = &ctx.accounts.payer;
    let pc_mint = &ctx.accounts.pc_mint;
    let pc_wallet = &mut ctx.accounts.pc_wallet;
    let scheduler_program = &ctx.accounts.scheduler_program;
    let system_program = &ctx.accounts.system_program;

    // Get remaining Accounts
    let swap_fee = ctx.remaining_accounts.get(0).unwrap();
    let swap_queue = ctx.remaining_accounts.get(1).unwrap();
    let swap_task = ctx.remaining_accounts.get(2).unwrap();
    let market = ctx.remaining_accounts.get(3).unwrap();
    let coin_vault = ctx.remaining_accounts.get(4).unwrap();
    let pc_vault = ctx.remaining_accounts.get(5).unwrap();
    let request_queue = ctx.remaining_accounts.get(6).unwrap();
    let event_queue = ctx.remaining_accounts.get(7).unwrap();
    let market_bids = ctx.remaining_accounts.get(8).unwrap();
    let market_asks = ctx.remaining_accounts.get(9).unwrap();
    let open_orders = ctx.remaining_accounts.get(10).unwrap();

    // get authority bump
    let bump = *ctx.bumps.get("authority").unwrap();

    // Create queue
    cronos_scheduler::cpi::queue_new(
        CpiContext::new_with_signer(
            scheduler_program.to_account_info(),
            cronos_scheduler::cpi::accounts::QueueNew {
                authority: authority.to_account_info(),
                clock: clock.to_account_info(),
                fee: swap_fee.to_account_info(),
                manager: manager.to_account_info(),
                payer: payer.to_account_info(),
                queue: swap_queue.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[&[SEED_AUTHORITY, authority.payer.as_ref(), &[bump]]],
        ),
        "*/30 * * * * * *".into(),
    )?;

    // create swap ix
    let swap_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority.key(), false),
            AccountMeta::new_readonly(dex_program.key(), false),
            AccountMeta::new(manager.key(), true),
            AccountMeta::new(cronos_scheduler::payer::ID, true),
            AccountMeta::new_readonly(pc_mint.key(), false),
            AccountMeta::new(pc_wallet.key(), false),
            AccountMeta::new_readonly(token::ID, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
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
        data: cronos_scheduler::anchor::sighash("swap").into(),
    };

    // Create task with the disburse ix and add it to the queue
    cronos_scheduler::cpi::task_new(
        CpiContext::new_with_signer(
            scheduler_program.to_account_info(),
            cronos_scheduler::cpi::accounts::TaskNew {
                authority: authority.to_account_info(),
                manager: manager.to_account_info(),
                payer: payer.to_account_info(),
                queue: swap_queue.to_account_info(),
                system_program: system_program.to_account_info(),
                task: swap_task.to_account_info(),
            },
            &[&[SEED_AUTHORITY, authority.payer.as_ref(), &[bump]]],
        ),
        vec![swap_ix.into()],
    )?;

    Ok(())
}
