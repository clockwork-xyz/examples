use {crate::state::*, anchor_lang::prelude::*};

#[derive(Accounts)]
pub struct InvestmentDelete<'info> {
    /// The authority (owner) of the investment.
    #[account()]
    pub authority: Signer<'info>,

    /// The address to return the data rent lamports to.
    #[account(mut)]
    pub close_to: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [
            SEED_INVESTMENT,
            investment.authority.as_ref(),
            investment.market.as_ref(),
        ],
        bump,
        has_one = authority,
        close = close_to
    )]
    pub investment: Account<'info, Investment>,
}

pub fn handler(_ctx: Context<InvestmentDelete>) -> Result<()> {
    Ok(())
}
