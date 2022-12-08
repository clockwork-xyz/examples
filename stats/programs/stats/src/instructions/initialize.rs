use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::system_program},
    std::mem::size_of,
};
#[derive(Accounts)]
#[instruction(lookback_window: i64, sample_rate: i64)]
pub struct Initialize<'info> {
    /// CHECK: this account should be a pyth feed account
    pub price_feed: AccountInfo<'info>,

    #[account(
        init,
        seeds = [
            SEED_STAT, 
            price_feed.key().as_ref(), 
            signer.key().as_ref(),
            &lookback_window.to_le_bytes(),
        ],
        bump,
        payer = signer,
        space = 8 + size_of::<Stat>(),
    )]
    pub stat: AccountLoader<'info, Stat>,

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<Initialize<'info>>, lookback_window: i64, sample_rate: i64) -> Result<()> {
    let price_feed = &ctx.accounts.price_feed;
    let signer = &ctx.accounts.signer;
    let stat = &mut ctx.accounts.stat.load_init()?;

    stat.new(price_feed.key(), signer.key(), lookback_window, sample_rate)?;

    Ok(())
}
