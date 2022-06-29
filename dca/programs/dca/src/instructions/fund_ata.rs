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
pub struct FundAta<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
      mut,
      associated_token::authority = buyer,
      associated_token::mint = usdc_mint,
    )]
    pub buyer_usdc_ata: Account<'info, TokenAccount>,

    #[account()]
    pub fund: Account<'info, Fund>,

    #[account(
      associated_token::authority = fund,
      associated_token::mint = usdc_mint,
    )]
    pub fund_usdc_ata: Account<'info, TokenAccount>,

    #[account(
      constraint = order.fund == fund.key()
    )]
    pub order: Account<'info, Order>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, FundAta<'info>>) -> Result<()> {
    // Get accounts
    let buyer = &mut ctx.accounts.buyer;
    let buyer_usdc_ata = &ctx.accounts.buyer_usdc_ata;
    let fund_usdc_ata = &ctx.accounts.fund_usdc_ata;
    let order = &ctx.accounts.order;
    let token_program = &ctx.accounts.token_program;

    // transfer funds form buyer usdc ata to fund usdc ata
    token::transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: buyer_usdc_ata.to_account_info().clone(),
                to: fund_usdc_ata.to_account_info().clone(),
                authority: buyer.to_account_info().clone(),
            },
        ),
        order.amount,
    )?;

    Ok(())
}
