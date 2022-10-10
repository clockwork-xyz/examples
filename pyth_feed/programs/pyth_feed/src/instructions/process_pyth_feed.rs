use {
    crate::state::*,
    anchor_lang::prelude::*,
    clockwork_sdk::queue_program::accounts::{Queue, QueueAccount},
    pyth_sdk_solana::load_price_feed_from_account_info,
};

#[derive(Accounts)]
pub struct ProcessPythFeed<'info> {
    #[account(mut, seeds = [SEED_FEED], bump)]
    pub feed: Account<'info, Feed>,

    /// CHECK: this account is manually being checked against the feed account's feed field
    #[account(
        constraint = pyth_data_feed.key() == feed.price_feed
    )]
    pub pyth_data_feed: AccountInfo<'info>,

    #[account(
        address = queue.pubkey(),
        constraint = queue.id.eq("events"),
        signer,
        constraint = queue.authority == feed.key()
    )]
    pub queue: Account<'info, Queue>,
}

pub fn handler<'info>(ctx: Context<ProcessPythFeed>) -> Result<()> {
    let feed = &mut ctx.accounts.feed;
    let pyth_data_feed = &ctx.accounts.pyth_data_feed;

    // load price feed
    let price_feed = load_price_feed_from_account_info(&pyth_data_feed.to_account_info()).unwrap();

    // update last publish time
    feed.publish_time = price_feed.publish_time;

    match price_feed.get_current_price() {
        Some(price) => {
            msg!(
                "Price change for {}: {}",
                price_feed.product_id.to_string(),
                price.price
            );
        }
        None => {}
    }

    Ok(())
}
