use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{system_program, sysvar},
    },
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{Mint, TokenAccount},
    },
    std::mem::size_of,
};

#[derive(Accounts)]

pub struct CreateFund<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account()]
    pub coin_mint: Account<'info, Mint>,

    #[account(
        init,
        seeds = [SEED_FUND, payer.key().as_ref()],
        bump,
        payer = payer,
        space = 8 + size_of::<Fund>(),
    )]
    pub fund: Account<'info, Fund>,

    #[account(
        init,
        payer = payer,
        associated_token::authority = fund,
        associated_token::mint = coin_mint
    )]
    pub fund_coin_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        associated_token::authority = fund,
        associated_token::mint = pc_mint
    )]
    pub fund_pc_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account()]
    pub pc_mint: Account<'info, Mint>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, CreateFund<'info>>) -> Result<()> {
    // Get accounts
    let fund = &mut ctx.accounts.fund;
    let payer = &mut ctx.accounts.payer;

    // initialize Fund
    fund.new(payer.key())?;

    Ok(())
}
