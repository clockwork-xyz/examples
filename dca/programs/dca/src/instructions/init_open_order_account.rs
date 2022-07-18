use anchor_spl::dex::serum_dex::state::Market;
use cronos_scheduler::state::Manager;

use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{system_program, sysvar},
    },
    anchor_spl::dex::{serum_dex::state::OpenOrders, InitOpenOrders},
    std::mem::size_of,
};

#[derive(Accounts)]
pub struct InitOpenOrderAccount<'info> {
    #[account(
        seeds = [SEED_AUTHORITY],
        bump
    )]
    pub authority: Account<'info, Authority>,

    #[account(address = anchor_spl::dex::ID)]
    pub dex_program: Program<'info, anchor_spl::dex::Dex>,

    #[account(has_one = authority)]
    pub manager: Account<'info, Manager>,

    #[account()]
    pub open_orders: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, InitOpenOrderAccount<'info>>) -> Result<()> {
    // Get accounts
    let authority = &ctx.accounts.authority;
    let dex_program = &ctx.accounts.dex_program;
    let open_orders = &ctx.accounts.open_orders;
    let rent = &ctx.accounts.rent;
    let _manager = &ctx.accounts.manager;

    // Get remaining accounts
    let market = ctx.remaining_accounts.get(0).unwrap();

    msg!("INIT OPEN ORDER ACCOUNT");

    let bump = *ctx.bumps.get("authority").unwrap();

    anchor_spl::dex::init_open_orders(CpiContext::new_with_signer(
        dex_program.to_account_info(),
        InitOpenOrders {
            authority: authority.to_account_info(),
            market: market.to_account_info(),
            open_orders: open_orders.to_account_info(),
            rent: rent.to_account_info(),
        },
        &[&[SEED_AUTHORITY, &[bump]]],
    ))?;

    Ok(())
}
