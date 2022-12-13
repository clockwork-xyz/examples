use {
    crate::state::*,
    anchor_lang::prelude::*,
    clockwork_sdk::thread_program::accounts::{Thread, ThreadAccount},
    pyth_sdk_solana::load_price_feed_from_account_info,
    std::{collections::VecDeque, cell::{RefMut, RefCell}},
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
    let mut stat = ctx.accounts.stat.load_mut()?;
    let stat_data = ctx.accounts.stat.as_ref().try_borrow_mut_data()?;
    
    let mut price_queue: VecDeque<Price> = VecDeque::new();
    let price_queue_rc = RefCell::new(&mut price_queue);
    let refmut_pq: RefMut<&mut VecDeque<Price>> = price_queue_rc.borrow_mut();

    load_entries_mut::<Stat, Price>(stat_data, refmut_pq)?;

    match load_price_feed_from_account_info(&price_feed.to_account_info()) {
        Ok(price_feed) => { 

            let price = price_feed.get_price_unchecked();

            stat.twap(Price { price: price.price, timestamp: price.publish_time }, &mut price_queue)?;

            let newest_price = price_queue.get((stat.sample_count - 1).try_into().unwrap()).unwrap();
            let oldest_price = price_queue
                .get(0)
                .unwrap();

            msg!("------------LIVE DATA------------");
            msg!("     live price: {}", price.price);
            msg!("      live time: {}", price.publish_time);
            msg!("--------STATS ACCOUNT DATA-------");
            msg!("     price feed: {}", stat.price_feed);
            msg!("      authority: {}", stat.authority);
            // msg!("     oldest - ts: {}, price: {}", oldest_price.timestamp, oldest_price.price);
            // msg!("     newest - ts: {}, price: {}", newest_price.timestamp, newest_price.price);
            msg!("      authority: {}", stat.authority);
            msg!("      TWA Price: {}", stat.twap);
            msg!(" lookback window: {} seconds", stat.lookback_window);
            msg!("    sample rate: {}", stat.sample_rate);
            msg!("   sample count: {}", stat.sample_count);
            msg!("     sample sum: {}", stat.sample_sum);
            msg!("           tail: {}", stat.tail);
            msg!("           head: {}", stat.head);
            msg!("---------------------------------");
        },
        Err(_) => {},
    }
    Ok(())
}
