use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::system_program},
    std::mem::size_of,
};

static INITIAL_BUFFER_LIMIT: usize = 100;

#[derive(Accounts)]
#[instruction(lookback_window: i64)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds = [
            SEED_AVG_BUFFER,
            stat.key().as_ref(), 
        ],
        bump,
        payer = signer,
        space = 8 + size_of::<AvgBuffer>() + (INITIAL_BUFFER_LIMIT * size_of::<i64>()),
    )]
    pub avg_buffer: AccountLoader<'info, AvgBuffer>,
    
    #[account(
        init,
        seeds = [
            SEED_PRICE_BUFFER,
            stat.key().as_ref(), 
        ],
        bump,
        payer = signer,
        space = 8 + size_of::<PriceBuffer>() + (INITIAL_BUFFER_LIMIT * size_of::<i64>()),
    )]
    pub price_buffer: AccountLoader<'info, PriceBuffer>,
    
    /// CHECK: this account should be a pyth feed account
    pub price_feed: AccountInfo<'info>,
    
    #[account(mut)]
    pub signer: Signer<'info>,

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
    pub stat: Account<'info, Stat>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(
        init,
        seeds = [
            SEED_TIME_SERIES, 
            stat.key().as_ref(), 
        ],
        bump,
        payer = signer,
        space = 8 + size_of::<TimeSeries>() + (INITIAL_BUFFER_LIMIT * size_of::<i64>()),
    )]
    pub time_series: AccountLoader<'info, TimeSeries>,
}

pub fn handler<'info>(ctx: Context<Initialize<'info>>, lookback_window: i64) -> Result<()> {
    let mut _avg_buffer= ctx.accounts.avg_buffer.load_init()?;
    let mut _price_buffer= ctx.accounts.price_buffer.load_init()?;
    let price_feed = &ctx.accounts.price_feed;
    let signer = &ctx.accounts.signer;
    let stat = &mut ctx.accounts.stat;
    let mut _time_series = ctx.accounts.time_series.load_init()?;

    stat.new(price_feed.key(), signer.key(), lookback_window, INITIAL_BUFFER_LIMIT)?;
    
    Ok(())
}
