use {
    crate::state::*,
    anchor_lang::prelude::*,
    clockwork_sdk::thread_program::accounts::{Thread, ThreadAccount},
    pyth_sdk_solana::load_price_feed_from_account_info,
};

#[derive(Accounts)]
pub struct Calc<'info> {
    #[account(
        mut,
        seeds = [
            SEED_STAT, 
            stat.load()?.price_feed.as_ref(), 
            stat.load()?.authority.as_ref(),
            &stat.load()?.lookback_window.to_le_bytes(),
        ],
        bump
    )]
    pub stat: AccountLoader<'info, Stat>,

    /// CHECK: this account is manually being checked against the stat account's price_feed field
    #[account(
        constraint = price_feed.key() == stat.load()?.price_feed
    )]
    pub price_feed: AccountInfo<'info>,

    #[account(
        constraint = thread.authority == stat.load()?.authority,
        address = thread.pubkey(),
        signer
    )]
    pub thread: Account<'info, Thread>,
}

pub fn handler<'info>(ctx: Context<Calc<'info>>) -> Result<()> {
    let price_feed = &ctx.accounts.price_feed;
    let stat = &mut ctx.accounts.stat.load_mut().unwrap();

    match load_price_feed_from_account_info(&price_feed.to_account_info()) {
        Ok(price_feed) => { 
            // get price unchecked
            let price = price_feed.get_price_unchecked();
            // calculate time weighted average
            stat.twap(price.publish_time, price.price)?;

            msg!("     price feed: {}", stat.price_feed);
            msg!("      authority: {}", stat.authority);
            msg!("      TWA Price: {}", stat.twap);
            msg!(" lookback window: {} seconds", stat.lookback_window);
            msg!("    sample rate: {}", stat.sample_rate);
            msg!("   sample count: {}", stat.sample_count);
            msg!("     sample sum: {}", stat.sample_sum);
        },
        Err(_) => {},
    }
    Ok(())
}
