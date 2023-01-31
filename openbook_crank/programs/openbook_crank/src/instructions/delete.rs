use {crate::state::*, anchor_lang::prelude::*};

#[derive(Accounts)]
pub struct Delete<'info> {
    #[account()]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub close_to: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [
            SEED_CRANK, 
            crank.authority.as_ref(), 
            crank.market.as_ref(), 
        ],
        bump,
        has_one = authority,
        close = close_to
    )]
    pub crank: Account<'info, Crank>,
}

pub fn handler(_ctx: Context<Delete>) -> Result<()> {
    Ok(())
}
