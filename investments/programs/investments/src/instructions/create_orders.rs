use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{system_program, sysvar},
    },
    anchor_spl::dex::InitOpenOrders,
};

#[derive(Accounts)]
pub struct CreateOrders<'info> {
    #[account(address = anchor_spl::dex::ID)]
    pub dex_program: Program<'info, anchor_spl::dex::Dex>,

    #[account(
        seeds = [
            SEED_INVESTMENT, 
            investment.payer.key().as_ref(), 
            investment.mint_a.key().as_ref(), 
            investment.mint_b.key().as_ref()
        ], 
        bump
    )]
    pub investment: Account<'info, Investment>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, CreateOrders<'info>>) -> Result<()> {
    // Get accounts
    let dex_program = &ctx.accounts.dex_program;
    let investment = &ctx.accounts.investment;
    let rent = &ctx.accounts.rent;

    // Get remaining accounts
    let market = ctx.remaining_accounts.get(0).unwrap();
    let open_orders = ctx.remaining_accounts.get(1).unwrap();

    // get investment bump
    let bump = *ctx.bumps.get("investment").unwrap();

    // make cpi to serum dex to init open order account
    anchor_spl::dex::init_open_orders(CpiContext::new_with_signer(
        dex_program.to_account_info(),
        InitOpenOrders {
            authority: investment.to_account_info(),
            market: market.to_account_info(),
            open_orders: open_orders.to_account_info(),
            rent: rent.to_account_info(),
        },
        &[&[
            SEED_INVESTMENT,
            investment.payer.as_ref(),
            investment.mint_a.as_ref(),
            investment.mint_b.as_ref(),
            &[bump],
        ]],
    ))?;

    Ok(())
}
