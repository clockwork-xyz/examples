use {
    anchor_lang::{
        prelude::*,
        solana_program::{system_program, sysvar},
    },
    anchor_spl::dex::InitOpenOrders,
    cronos_scheduler::state::{Manager, SEED_MANAGER},
};

#[derive(Accounts)]
pub struct InitOOAccount<'info> {
    #[account(address = anchor_spl::dex::ID)]
    pub dex_program: Program<'info, anchor_spl::dex::Dex>,

    #[account(seeds = [SEED_MANAGER, manager.authority.as_ref()], bump)]
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

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, InitOOAccount<'info>>) -> Result<()> {
    // Get accounts
    let dex_program = &ctx.accounts.dex_program;
    let manager = &ctx.accounts.manager;
    let open_orders = &ctx.accounts.open_orders;
    let rent = &ctx.accounts.rent;

    // Get remaining accounts
    let market = ctx.remaining_accounts.get(0).unwrap();

    let bump = *ctx.bumps.get("manager").unwrap();

    anchor_spl::dex::init_open_orders(CpiContext::new_with_signer(
        dex_program.to_account_info(),
        InitOpenOrders {
            authority: manager.to_account_info(),
            market: market.to_account_info(),
            open_orders: open_orders.to_account_info(),
            rent: rent.to_account_info(),
        },
        &[&[SEED_MANAGER, manager.authority.as_ref(), &[bump]]],
    ))?;

    Ok(())
}
