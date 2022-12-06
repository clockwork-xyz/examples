use {crate::state::*, anchor_lang::prelude::*};

#[derive(Accounts)]
pub struct Delete<'info> {
    #[account()]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SEED_STAT, 
            stat.load()?.price_feed.as_ref(), 
            stat.load()?.authority.as_ref(),
            &stat.load()?.lookback_window.to_le_bytes(),
        ],
        bump,
        constraint = stat.load()?.authority == authority.key(),
        close = authority
      )]
    pub stat: AccountLoader<'info, Stat>,
}

pub fn handler(_ctx: Context<Delete>) -> Result<()> {
    Ok(())
}
