use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{system_program, sysvar},
    },
    anchor_spl::token::{self, Mint, TokenAccount},
    std::mem::size_of,
};

#[derive(Accounts)]
#[instruction(swap_amount: u64)]
pub struct CreateInvestment<'info> {
    #[account(address = anchor_spl::dex::ID)]
    pub dex_program: Program<'info, OpenBookDex>,

    #[account(
        init,
        seeds = [SEED_INVESTMENT, payer.key().as_ref(), market.key().as_ref()],
        bump,
        payer = payer,
        space = 8 + size_of::<Investment>(),
    )]
    pub investment: Account<'info, Investment>,

    /// CHECK:
    pub market: AccountInfo<'info>,

    /// CHECK:
    pub mint_a: Account<'info, Mint>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        associated_token::authority = payer,
        associated_token::mint = mint_a,
    )]
    pub payer_mint_a_token_account: Account<'info, TokenAccount>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, CreateInvestment<'info>>,
    swap_amount: u64,
) -> Result<()> {
    // Get accounts
    let dex_program = &ctx.accounts.dex_program;
    let investment = &mut ctx.accounts.investment;
    let market = &ctx.accounts.market;
    let payer = &ctx.accounts.payer;
    let payer_mint_a_token_account = &mut ctx.accounts.payer_mint_a_token_account;
    let rent = &ctx.accounts.rent;
    let token_program = &ctx.accounts.token_program;

    // Get remaining accounts
    let open_orders = ctx.remaining_accounts.get(0).unwrap();

    // get investment bump
    let bump = *ctx.bumps.get("investment").unwrap();

    // initialize investment account
    investment.new(payer.key(), swap_amount, market.key())?;

    // Approve the investment account to spend from the authority's token account.
    token::approve(
        CpiContext::new(
            token_program.to_account_info(),
            token::Approve {
                to: payer_mint_a_token_account.to_account_info(),
                delegate: investment.to_account_info(),
                authority: payer.to_account_info(),
            },
        ),
        u64::MAX,
    )?;

    //init open order account
    anchor_spl::dex::init_open_orders(CpiContext::new_with_signer(
        dex_program.to_account_info(),
        anchor_spl::dex::InitOpenOrders {
            authority: investment.to_account_info(),
            market: market.to_account_info(),
            open_orders: open_orders.to_account_info(),
            rent: rent.to_account_info(),
        },
        &[&[
            SEED_INVESTMENT,
            investment.payer.as_ref(),
            investment.market.as_ref(),
            &[bump],
        ]],
    ))?;

    Ok(())
}
