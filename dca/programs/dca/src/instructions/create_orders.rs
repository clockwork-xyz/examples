use {    
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{system_program, sysvar},
    },
    anchor_spl::dex::InitOpenOrders
};

#[derive(Accounts)]
pub struct CreateOrders<'info> {
    #[account( 
        seeds = [SEED_AUTHORITY, authority.payer.as_ref()],
        bump
    )]
    pub authority: Account<'info, Authority>,

    #[account(address = anchor_spl::dex::ID)]
    pub dex_program: Program<'info, anchor_spl::dex::Dex>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, CreateOrders<'info>>) -> Result<()> {
    // Get accounts
    let authority = &ctx.accounts.authority;
    let dex_program = &ctx.accounts.dex_program;
    let rent = &ctx.accounts.rent;

    // Get remaining accounts
    let market = ctx.remaining_accounts.get(0).unwrap();
    let open_orders = ctx.remaining_accounts.get(1).unwrap();

    // get authority bump
    let bump = *ctx.bumps.get("authority").unwrap();

    // make cpi to serum dex to init open order account
    anchor_spl::dex::init_open_orders(CpiContext::new_with_signer(
        dex_program.to_account_info(),
        InitOpenOrders {
            authority: authority.to_account_info(),
            market: market.to_account_info(),
            open_orders: open_orders.to_account_info(),
            rent: rent.to_account_info(),
        },
        &[&[SEED_AUTHORITY, authority.payer.as_ref(), &[bump]]],
    ))?;

    Ok(())
}
