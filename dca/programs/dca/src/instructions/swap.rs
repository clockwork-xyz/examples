use anchor_spl::token::TokenAccount;

use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::system_program},
    anchor_spl::{
        dex::serum_dex::state::{Market, OpenOrders},
        token::TokenAccount,
    },
    std::mem::size_of,
};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(address == anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        init,
        payer = buyer,
        associated_token::mint = index_token_mint,
        associated_token::authority = buyer
    )]
    pub buyer_index_ata: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub buyer_usdc_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = order.fund.key() == fund.key()
    )]
    pub fund: Account<'info, Fund>,

    #[account(mut)]
    pub fund_usdc_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub index_token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub order: Account<'info, Order>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, Token>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::ID)]
    pub dex_program: Program<'info, anchor_spl::dex::Dex>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, Swap<'info>>) -> Result<()> {
    // Get accounts
    let associated_token_program = &ctx.accounts.associated_token_program;
    let buyer = &ctx.accounts.buyer;
    let buyer_index_ata = &ctx.accounts.buyer_index_ata;
    let buyer_usdc_ata = &ctx.accounts.buyer_usdc_ata;
    let dex_program = &ctx.accounts.dex_program;
    let fund = &mut ctx.accounts.fund;
    let fund_usdc_ata = &ctx.accounts.fund_usdc_ata;
    let index_mint = &ctx.accounts.index_token_mint;
    let order = &mut ctx.accounts.order;
    let token_program = &ctx.accounts.token_program;
    let system_program = &ctx.accounts.system_program;
    let rent_sysvar = &ctx.accounts.rent;

    // Get remaining accounts
    let remaining_accounts_iter = &mut ctx.remaining_accounts.iter();
    let market = next_account_info(remaining_accounts_iter)?;
    let open_orders = next_account_info(remaining_accounts_iter)?;
    let request_queue = next_account_info(remaining_accounts_iter)?;
    let event_queue = next_account_info(remaining_accounts_iter)?;
    let bids = next_account_info(remaining_accounts_iter)?;
    let asks = next_account_info(remaining_accounts_iter)?;
    let coin_vault = next_account_info(remaining_accounts_iter)?;
    let pc_vault = next_account_info(remaining_accounts_iter)?;
    let vault_signer = next_account_info(remaining_accounts_iter)?;
    let coin_wallet = next_account_info(remaining_accounts_iter)?;
    let asset_mint = next_account_info(remaining_accounts_iter)?;

    // get fund asset ata
    let fund_asset_ata = get_associated_token_address(
        &fund.to_account_info().key(),
        &asset_mint.to_account_info().key(),
    );

    // create coin vault ata if needed
    if coin_wallet.to_account_info().data_is_empty() {
        create_ata(
            &buyer.to_account_info(),
            &fund.to_account_info(),
            &asset_mint.to_account_info(),
            &coin_wallet.to_account_info(),
            &token_program.to_account_info(),
            &associated_token_program.to_account_info(),
            &system_program.to_account_info(),
            &rent_sysvar.to_account_info(),
        )?;
    }

    serum_swap::cpi::swap(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            serum_swap::Swap {
                authority: order.fund,
                market,
                pc_wallet,
                dex_program,
                token_program,
                rent,
            },
            &[&[&[fund.manager.as_ref(), fund.name.as_ref(), SEED_FUND]]],
        ),
        order.side,
        order.amount,
        min_exchange_rate,
    )?;

    Ok(())
}
