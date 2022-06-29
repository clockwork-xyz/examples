use crate::instruction;

use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::system_program},
    anchor_spl::token::{Mint, TokenAccount},
    std::mem::size_of,
};

#[derive(Accounts)]
#[instruction(
    name: String, 
    symbol: String, 
    assets: [Pubkey; 3], 
    weights [u64, 3], 
    token_decimals: u8
)]
pub struct CreateFund<'info> {

    #[account(address == anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(address = anchor_spl::ID)]
    pub dex_program: Program<'info, anchor_spl::dex::Dex>,

    #[account(
        init,
        seeds = [SEED_FUND, manager.key().as_ref(), name.as_ref()],
        bump,
        payer = manager,
        space = 8 + size_of::<Fund>(),
    )]
    pub fund: Account<'info, Fund>,

     #[account(
        init,
        payer = manager,
        associated_token::mint = usdc_mint,
        associated_token::authority = fund
    )]
    pub fund_usdc_ata: Account<'info, TokenAccount>,

    #[account(
        init,
        seeds = [manager.key().as_ref(), name.as_ref()],
        bump,
        payer = manager,
        mint::decimals = token_decimals,
        mint::authority = index_token_mint
    )]
    pub index_token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub manager: Signer<'info>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,

    #[account()]
    pub usdc_mint: Account<'info, Mint>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, CreateFund<'info>>,
    name: String,
    symbol: String,
    assets: [Pubkey; 3],
    weights: [u64; 3],
    _token_decimals: u8
) -> Result<()> {
    
    // Get accounts
    let dex_program = &ctx.accounts.dex_program;
    let fund = &mut ctx.accounts.fund;
    let fund_usdc_ata = &ctx.accounts.fund_usdc_ata;
    let index_token_mint = &ctx.accounts.index_token_mint;
    let manager = &ctx.accounts.manager;
    let rent = &ctx.accounts.rent;
    let system_program = &ctx.accounts.system_program;
    let token_program = &ctx.accounts.token_program;
    let usdc_mint = &ctx.accounts.usdc_mint;

    // Get remaining accounts
    let market = &ctx.remaining_accounts.get(0).unwrap();
    let open_orders = &ctx.remaining_accounts.get(1).unwrap();

    let fund_bump = ctx.bumps.get("fund").unwrap();

    // initialize Fund
    fund.new(
        name,
        symbol,
        manager.key(),
        assets,
        weights,
        index_token_mint.key(),
    )?;

    // init open order account for newly initialized fund
    serum_swap::cpi::init_account(CpiContext::new_with_signer(
        serum_swap_program.to_account_info(),
        serum_swap::cpi::accounts::InitAccount {
            authority: fund.key(),
            dex_program: dex_program.key(),
            market: market.key(),
            open_orders: open_orders.key(),
            rent: rent.key(),
        },
        &[&[
            SEED_FUND,
            manager.key().as_ref(),
            name.key().as_ref(),
            &[fund_bump],
        ]]
    ))?;

    Ok(())
}
