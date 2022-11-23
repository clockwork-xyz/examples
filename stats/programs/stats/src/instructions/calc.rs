use {
    crate::objects::*,
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
            stat.price_feed.as_ref(), 
            stat.authority.as_ref()
        ],
        bump,
    )]
    pub stat: Account<'info, Stat>,

    /// CHECK: this account is manually being checked against the stat account's price_feed field
    #[account(
        constraint = price_feed.key() == stat.price_feed
    )]
    pub price_feed: AccountInfo<'info>,

    #[account(
        address = thread.pubkey(),
        constraint = thread.id.eq("stats"),
        signer,
        constraint = thread.authority == stat.authority
    )]
    pub thread: Account<'info, Thread>,
}

pub fn handler<'info>(ctx: Context<Calc<'info>>) -> Result<()> {
    let stat = &mut ctx.accounts.stat;
    let price_feed = &ctx.accounts.price_feed;

    match load_price_feed_from_account_info(&price_feed.to_account_info()) {
        Ok(price_feed) => {
            // get price unchecked
            let price = price_feed.get_price_unchecked();
            // calculate time weighted average
            stat.twap(price.publish_time, price.price)?;

            msg!(
                "TWA Price: {} for lookback window: {}",
                stat.twap,
                stat.lookback_window
            );
        }
        Err(_) => {}
    }
    Ok(())
}
