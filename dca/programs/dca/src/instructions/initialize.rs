use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::system_program},
    anchor_spl::dex::serum_dex::state::{Market, OpenOrders},
    std::mem::size_of,
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds = [SEED_AUTHORITY],
        bump,
        payer = payer,
        space = 8 + size_of::<Authority>(),
    )]
    pub authority: Account<'info, Authority>,

    #[account(address = anchor_spl::ID)]
    pub dex_program: Program<'info, anchor_spl::dex::Dex>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = cronos_scheduler::ID)]
    pub scheduler_program: Program<'info, cronos_scheduler::program::CronosScheduler>,

    #[account(address = serum_swap::ID)]
    pub swap_program: Program<'info, serum_swap::program::SerumSwap>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, Initialize<'info>>) -> Result<()> {
    // Get accounts
    let authority = &mut ctx.accounts.authority;
    let dex_program = &ctx.accounts.dex_program;
    let payer = &mut ctx.accounts.payer;
    let rent = &ctx.accounts.rent;
    let scheduler_program = &ctx.accounts.scheduler_program;
    let swap_program = &ctx.accounts.swap_program;
    let system_program = &ctx.accounts.system_program;

    // Get remaining accounts
    let manager = ctx.remaining_accounts.get(0).unwrap();
    let market = ctx.remaining_accounts.get(1).unwrap();
    let open_orders = ctx.remaining_accounts.get(2).unwrap();

    // initialize accounts
    authority.new(manager.key())?;

    // init serum's open order
    serum_swap::cpi::init_account(CpiContext::new(
        serum_swap_program.to_account_info(),
        serum_swap::cpi::accounts::InitAccount {
            authority: payer.key(),
            dex_program: dex_program.key(),
            market: market.key(),
            open_orders: open_orders.key(),
            rent: rent.key(),
        },
    ))?;

    // create manager
    let bump = *ctx.bumps.get("authority").unwrap();
    cronos_scheduler::cpi::manager_new(CpiContext::new_with_signer(
        scheduler_program.to_account_info(),
        cronos_scheduler::cpi::accounts::ManagerNew {
            authority: authority.to_account_info(),
            manager: manager.to_account_info(),
            payer: payer.to_account_info(),
            system_program: system_program.to_account_info(),
        },
        &[&[SEED_AUTHORITY, &[bump]]],
    ))?;

    Ok(())
}
