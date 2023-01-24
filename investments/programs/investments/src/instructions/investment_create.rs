use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{system_program, sysvar},
    },
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{self, Mint, TokenAccount},
    },
    std::mem::size_of,
};

#[derive(Accounts)]
#[instruction(swap_amount: u64)]
pub struct InvestmentCreate<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        associated_token::authority = authority,
        associated_token::mint = mint_a,
    )]
    pub authority_mint_a_vault: Account<'info, TokenAccount>,

    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(address = anchor_spl::dex::ID)]
    pub dex_program: Program<'info, OpenBookDex>,

    #[account(
        init,
        seeds = [SEED_INVESTMENT, authority.key().as_ref(), market.key().as_ref()],
        bump,
        payer = authority,
        space = 8 + size_of::<Investment>(),
    )]
    pub investment: Account<'info, Investment>,

    #[account(
        init,
        payer = authority,
        associated_token::mint = mint_a,
        associated_token::authority = investment
    )]
    pub investment_mint_a_vault: Account<'info, TokenAccount>,

    /// CHECK:
    pub market: AccountInfo<'info>,

    /// CHECK:
    pub mint_a: Account<'info, Mint>,

    /// CHECK:
    pub mint_b: Account<'info, Mint>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, InvestmentCreate<'info>>,
    swap_amount: u64,
) -> Result<()> {
    // Get accounts
    let authority = &ctx.accounts.authority;
    let authority_mint_a_vault = &mut ctx.accounts.authority_mint_a_vault;
    let dex_program = &ctx.accounts.dex_program;
    let investment = &mut ctx.accounts.investment;
    let _investment_mint_a_vault = &ctx.accounts.investment_mint_a_vault;
    let market = &ctx.accounts.market;
    let mint_a = &ctx.accounts.mint_a;
    let mint_b = &ctx.accounts.mint_b;
    let rent = &ctx.accounts.rent;
    let token_program = &ctx.accounts.token_program;

    // Get remaining accounts
    let open_orders = ctx.remaining_accounts.get(0).unwrap();

    // initialize investment account
    investment.new(
        authority.key(),
        market.key(),
        mint_a.key(),
        mint_b.key(),
        swap_amount,
    )?;

    // Approve the investment account to spend from the authority's token account.
    token::approve(
        CpiContext::new(
            token_program.to_account_info(),
            token::Approve {
                to: authority_mint_a_vault.to_account_info(),
                delegate: investment.to_account_info(),
                authority: authority.to_account_info(),
            },
        ),
        u64::MAX,
    )?;

    // init open order account
    anchor_spl::dex::init_open_orders(CpiContext::new(
        dex_program.to_account_info(),
        anchor_spl::dex::InitOpenOrders {
            authority: investment.to_account_info(),
            market: market.to_account_info(),
            open_orders: open_orders.to_account_info(),
            rent: rent.to_account_info(),
        },
    ))?;

    Ok(())
}
