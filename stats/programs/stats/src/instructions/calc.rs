use {
    crate::state::*,
    anchor_lang::{prelude::*, Discriminator, solana_program::system_program},
    bytemuck::{Pod, Zeroable},
    clockwork_sdk::thread_program::accounts::{Thread, ThreadAccount},
    pyth_sdk_solana::load_price_feed_from_account_info,
    std::cell::{Ref, RefMut}
};

#[derive(Accounts)]
pub struct Calc<'info> {
    #[account(
        mut,
        seeds = [
            SEED_DATASET,
            stat.key().as_ref(),
        ],
        bump
    )]
    pub dataset: AccountLoader<'info, Dataset>,
    
    #[account(
        mut,
        seeds = [
            SEED_STAT, 
            stat.price_feed.as_ref(), 
            stat.authority.as_ref(),
            &stat.lookback_window.to_le_bytes(),
        ],
        bump
    )]
    pub stat: Account<'info, Stat>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: this account is manually being checked against the stat account's price_feed field
    #[account(
        constraint = price_feed.key() == stat.price_feed
    )]
    pub price_feed: AccountInfo<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(
        constraint = thread.authority == stat.authority,
        address = thread.pubkey(),
        signer
    )]
    pub thread: Account<'info, Thread>,
}

pub fn handler<'info>(ctx: Context<Calc<'info>>) -> Result<()> {
    let price_feed = &ctx.accounts.price_feed;
    let stat = &mut ctx.accounts.stat;
    let mut data_points = load_entries_mut::<Dataset, PriceData>(ctx.accounts.dataset.as_ref().try_borrow_mut_data()?).unwrap();

    match load_price_feed_from_account_info(&price_feed.to_account_info()) {
        Ok(price_feed) => { 
            // Load Pyth price fee. 
            let price = price_feed.get_price_unchecked();

            // Starting at the tail, nullify data points older than the lookback window.
            let mut tail = (stat.head + stat.sample_count - 1) % stat.buffer_limit;
            let mut oldest_data_point = data_points.get(tail);
            while oldest_data_point.is_some() && oldest_data_point.unwrap().ts < price.publish_time - stat.lookback_window {
                stat.sample_sum -= oldest_data_point.unwrap().price;
                stat.sample_count -= 1;
                tail = (tail - 1) % stat.buffer_limit;
                oldest_data_point = data_points.get(tail);

                // TODO This is a worst-case linear operation over a large dataset. 
                //      Watch out for exceeding compute unit limits. Since this is a threaded instruction,
                //      we can run it as an infinite loop until we've cleared out all the old data.
            }

            // Insert data point into ring buffer.
            stat.head = stat.head + 1 % stat.buffer_limit;
            if stat.head == tail {
                stat.sample_sum -= data_points.get(tail).unwrap().price;
            } else {
                stat.sample_count += 1;
            }
            data_points[stat.head] = PriceData { price: price.price, ts: price.publish_time };
            stat.sample_sum += price.price;

            // Compute new average.
            stat.sample_avg = stat.sample_sum.checked_div(stat.sample_count as i64).unwrap();

            // Price the latest stats.
            let tail = (stat.head + stat.sample_count - 1) % stat.buffer_limit;
            let newest_price = data_points.get(stat.head).unwrap();
            let oldest_price = data_points.get(tail).unwrap();
            msg!("------------LIVE DATA------------");
            msg!("     live price: {}", price.price);
            msg!("      live time: {}", price.publish_time);
            msg!("--------STATS ACCOUNT DATA-------");
            msg!("     price feed: {}", stat.price_feed);
            msg!("      authority: {}", stat.authority);
            msg!("    oldest - ts: {}, price: {}", oldest_price.ts, oldest_price.price);
            msg!("    newest - ts: {}, price: {}", newest_price.ts, newest_price.price);
            msg!("      authority: {}", stat.authority);
            msg!("      avg price: {}", stat.sample_avg);
            msg!("lookback window: {} seconds", stat.lookback_window);
            msg!("    sample rate: {}", stat.sample_rate);
            msg!("   sample count: {}", stat.sample_count);
            msg!("     sample sum: {}", stat.sample_sum);
            msg!("           head: {}", stat.head);
            msg!("           tail: {}", tail);
            msg!("---------------------------------");
        },
        Err(_) => {},
    }
    Ok(())
}

#[derive(Copy, Clone, Zeroable, Pod)]
#[repr(C)]
pub struct PriceData {
    pub price: i64,
    pub ts: i64,
}

#[inline(always)]
pub fn _load_entries<'a, THeader, TEntries>(data: Ref<'a, &mut [u8]>) -> Result<Ref<'a, [TEntries]>>
where
    THeader: Discriminator,
    TEntries: bytemuck::Pod,
{
    Ok(Ref::map(data, |data| {
        bytemuck::cast_slice(&data[8 + std::mem::size_of::<THeader>()..data.len()])
    }))
}

#[inline(always)]
pub fn load_entries_mut<'a, THeader, TEntries>(
    data: RefMut<'a, &mut [u8]>,
) -> Result<RefMut<'a, [TEntries]>>
where
    THeader: bytemuck::Pod + Discriminator,
    TEntries: bytemuck::Pod,
{
    Ok(RefMut::map(data, |data| {
        let len = data.len(); 
        bytemuck::cast_slice_mut::<u8, TEntries>(&mut data[8 + 8 + std::mem::size_of::<THeader>()..len])
    }))
}
