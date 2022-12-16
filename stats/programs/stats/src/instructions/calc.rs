use {
    crate::state::*,
    anchor_lang::{prelude::*, Discriminator, solana_program::system_program},
    bytemuck::{Pod, Zeroable},
    clockwork_sdk::{thread_program::accounts::{Thread, ThreadAccount}, ThreadResponse, InstructionData, AccountMetaData},
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

    /// CHECK: this account is manually being checked against the stat account's price_feed field
    #[account(address = stat.price_feed)]
    pub price_feed: AccountInfo<'info>,

    #[account(
        constraint = thread.authority == stat.authority,
        address = thread.pubkey(),
        signer
    )]
    pub thread: Account<'info, Thread>,
}

pub fn handler<'info>(ctx: Context<Calc<'info>>) -> Result<ThreadResponse> {
    let price_feed = &ctx.accounts.price_feed;
    let stat = &mut ctx.accounts.stat;
    // let thread = &ctx.accounts.thread;
    let dataset = ctx.accounts.dataset.as_ref();
    let mut data_points = load_entries_mut::<Dataset, PriceData>(dataset.try_borrow_mut_data()?).unwrap();

    let mut kickoff_instruction: Option<InstructionData> = None;
    let mut next_instruction: Option<InstructionData> = None;

    match load_price_feed_from_account_info(&price_feed.to_account_info()) {
        Ok(price_feed) => { 
               // Load Pyth price fee. 
            let price = price_feed.get_price_unchecked();

            let mut tail: i64 = 0;

            if stat.head.is_some() {                
                // increment head for next insertion if sample rate threshold has been met
                if price.publish_time >= stat.sample_rate + data_points.get(stat.head.unwrap() as usize).unwrap().ts {
                    stat.head = Some((stat.head.unwrap() + 1) % stat.buffer_limit as i64); // (4 + 1) % 5 = 0 
                    
                    // if buffer has begun rotating
                    if data_points.get(stat.head.unwrap() as usize).unwrap().ts > 0 {
                        // reset index for it to be the next insertion
                        stat.sample_sum -= data_points.get(stat.head.unwrap() as usize).unwrap().price;
                        data_points[stat.head.unwrap() as usize] = PriceData::default();
                    }
                    
                }
                // calculate tail
                tail = (stat.head.unwrap() - stat.sample_count as i64 + 1).rem_euclid(stat.buffer_limit as i64); // (0 - 5 + 1) % 5 = 1 
                msg!("tail calc: ({} - {} + 1) % {} = {}",stat.head.unwrap(), stat.sample_count as i64, stat.buffer_limit as i64, tail);
                // at this point the head and tail have been calculated and if the buffer
                // has begun rotating we've taken the steps necc. to get ready
                // for the next insertion.
            } else {
                stat.head = Some(0);
            }
            
            // let lookback_window_threshold: bool = data_points.get(tail).unwrap().ts < price.publish_time - stat.lookback_window;
            // Starting at the tail, nullify data points older than the lookback window.
            // while stat.sample_count > 0 && lookback_window_threshold {
            //     stat.sample_sum -= oldest_price.unwrap().price;
            //     stat.sample_count -= 1;
            //     data_points[tail] = PriceData::default();
            //     tail = (tail + 1) % stat.buffer_limit;
            //     oldest_price = data_points.get(tail);
                // TODO This is a worst-case linear operation over a large dataset. 
                //      Watch out for exceeding compute unit limits. Since this is a threaded instruction,
                //      we can run it as an infinite loop until we've cleared out all the old data.
            // }

            // if new data ts is after sample rate threashold or there are 0 elements
            if price.publish_time >= stat.sample_rate + data_points.get(stat.head.unwrap() as usize).unwrap().ts || stat.sample_count == 0 {
                // make sure sample count doesn't exceed buffer limit
                // TODO: REMOVE THIS CONDITION WHEN REINTRODUCING REALLOCATION OF DATASET ACCOUNT SIZE
                if stat.sample_count < stat.buffer_limit {
                    stat.sample_count += 1;
                }
                // insert new data into buffer
                data_points[stat.head.unwrap() as usize] = PriceData { price: price.price, ts: price.publish_time };
                // increase sample sum
                stat.sample_sum += price.price;
                // Compute new average.
                stat.sample_avg = stat.sample_sum.checked_div(stat.sample_count as i64).unwrap();
            }

            msg!("[0]: {}", data_points.get(0).unwrap().ts);
            msg!("[1]: {}", data_points.get(1).unwrap().ts);
            msg!("[2]: {}", data_points.get(2).unwrap().ts);
            msg!("[3]: {}", data_points.get(3).unwrap().ts);
            msg!("[4]: {}", data_points.get(4).unwrap().ts);

            // let new_realloc_size: usize = 8 + std::mem::size_of::<Dataset>() + ((stat.buffer_limit + 640) * std::mem::size_of::<crate::PriceData>());
            // if stat.sample_count == stat.buffer_limit && new_realloc_size < 10_000_000 {
            //     next_instruction = 
            //         Some(InstructionData { 
            //                 program_id: crate::ID, 
            //                 accounts: vec![
            //                     AccountMetaData::new(dataset.key(), false),
            //                     AccountMetaData::new(stat.key(), false),
            //                     AccountMetaData::new(clockwork_sdk::PAYER_PUBKEY, true),
            //                     AccountMetaData::new_readonly(system_program::ID, false),
            //                     AccountMetaData::new(thread.key(), true),
            //                 ], 
            //                 data: clockwork_sdk::anchor_sighash("realloc_buffer").to_vec() 
            //             });
            // } else {
            //     kickoff_instruction = Some(InstructionData {
            //         program_id: crate::ID,
            //         accounts: vec![
            //             AccountMetaData::new(dataset.key(), false),
            //             AccountMetaData::new(stat.key(), false),
            //             AccountMetaData::new_readonly(stat.price_feed, false),
            //             AccountMetaData::new(thread.key(), true),
            //         ],
            //         data: clockwork_sdk::anchor_sighash("calc").to_vec()
            //     }) 
            // }
            
            msg!("------------LIVE DATA------------");
            msg!("     live price: {}", price.price);
            msg!("      live time: {}", price.publish_time);
            msg!("--------STATS ACCOUNT DATA-------");
            msg!("     price feed: {}", stat.price_feed);
            msg!("      authority: {}", stat.authority);
            // msg!("    oldest - ts: {}", data_points.get(tail as usize).unwrap().ts); // TODO: ERROR HERE??
            // msg!("    newest - ts: {}", data_points.get(stat.head.unwrap() as usize).unwrap().ts);
            msg!("      avg price: {}", stat.sample_avg);
            msg!("lookback window: {} seconds", stat.lookback_window);
            msg!("    sample rate: {}", stat.sample_rate);
            msg!("   sample count: {}", stat.sample_count);
            msg!("     sample sum: {}", stat.sample_sum);
            msg!("   buffer_limit: {}", stat.buffer_limit);
            msg!("           head: {:?}", stat.head);
            msg!("           tail: {}", tail);
            msg!("---------------------------------");
        },
        Err(_) => {},
    }
    
    Ok(ThreadResponse { kickoff_instruction , next_instruction })
}

#[derive(Copy, Clone, Zeroable, Pod, Default)]
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
    THeader: Discriminator,
    TEntries: bytemuck::Pod,
{
    Ok(RefMut::map(data, |data| {
        let len = data.len();
        bytemuck::cast_slice_mut::<u8, TEntries>(&mut data[8 + std::mem::size_of::<THeader>()..len])
    }))
}
