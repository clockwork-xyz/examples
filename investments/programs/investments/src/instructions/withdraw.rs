use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{system_program, sysvar},
    },
    anchor_spl::token::{self, Mint, TokenAccount, Transfer},
};

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Withdraw<'info> {
    #[account(
        seeds = [SEED_INVESTMENT, investment.payer.as_ref(), investment.mint_a.as_ref(), investment.mint_b.as_ref()],
        bump,
        has_one = payer,
        has_one = mint_a
    )]
    pub investment: Account<'info, Investment>,

    #[account(
        mut,
        associated_token::authority = investment,
        associated_token::mint = investment.mint_a,
    )]
    pub investment_mint_a_token_accoount: Account<'info, TokenAccount>,

    #[account()]
    pub mint_a: Account<'info, Mint>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        associated_token::authority = investment.payer,
        associated_token::mint = investment.mint_a,
    )]
    pub payer_mint_a_token_account: Account<'info, TokenAccount>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = clockwork_crank::ID)]
    pub scheduler_program: Program<'info, clockwork_crank::program::ClockworkCrank>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, Withdraw<'info>>, amount: u64) -> Result<()> {
    // Get accounts
    let investment = &mut ctx.accounts.investment;
    let investment_mint_a_token_accoount = &mut ctx.accounts.investment_mint_a_token_accoount;
    let payer_mint_a_token_account = &mut ctx.accounts.payer_mint_a_token_account;
    let token_program = &ctx.accounts.token_program;

    // get investment bump
    let bump = *ctx.bumps.get("investment").unwrap();

    // deposit funds from sender's token account to escrow token account
    token::transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: investment_mint_a_token_accoount.to_account_info(),
                to: payer_mint_a_token_account.to_account_info(),
                authority: investment.to_account_info(),
            },
            &[&[
                SEED_INVESTMENT,
                investment.payer.as_ref(),
                investment.mint_a.as_ref(),
                investment.mint_b.as_ref(),
                &[bump],
            ]],
        ),
        amount,
    )?;

    Ok(())
}
