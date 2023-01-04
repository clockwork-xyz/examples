use {
    crate::state::*,
    anchor_lang::{prelude::*, Discriminator, solana_program::system_program},
    clockwork_sdk::state::{Thread, ThreadAccount, ThreadResponse, InstructionData, AccountMetaData},
    pyth_sdk_solana::load_price_feed_from_account_info,
    std::cell::{Ref, RefMut}
};

#[derive(Accounts)]
pub struct Calc<'info> {
     #[account(
        mut,
        seeds = [
            SEED_AVG_BUFFER,
            stat.key().as_ref(),
        ],
        bump
    )]
    pub avg_buffer: AccountLoader<'info, AvgBuffer>,

    #[account(
        mut,
        seeds = [
            SEED_PRICE_BUFFER,
            stat.key().as_ref(),
        ],
        bump
    )]
    pub price_buffer: AccountLoader<'info, PriceBuffer>,
    
    /// CHECK: this account is manually being checked against the stat account's price_feed field
    #[account(address = stat.price_feed)]
    pub price_feed: AccountInfo<'info>,

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

    #[account(
        constraint = thread.authority == stat.authority,
        address = thread.pubkey(),
        signer
    )]
    pub thread: Account<'info, Thread>,

    #[account(
        mut,
        seeds = [
            SEED_TIME_SERIES, 
            stat.key().as_ref(), 
        ],
        bump
    )]
    pub time_series: AccountLoader<'info, TimeSeries>,
}

pub fn handler<'info>(ctx: Context<Calc<'info>>) -> Result<ThreadResponse> {
    // get accounts
    let avg_buffer_acc = ctx.accounts.avg_buffer.as_ref();
    let price_buffer_acc = ctx.accounts.price_buffer.as_ref();
    let price_feed = &ctx.accounts.price_feed;
    let stat = &mut ctx.accounts.stat;
    let thread = &ctx.accounts.thread;
    let time_series_acc = ctx.accounts.time_series.as_ref();

    // load mut entries
    let mut avg_buffer = load_entries_mut::<AvgBuffer, i64>(avg_buffer_acc.try_borrow_mut_data()?).unwrap();
    let mut price_buffer = load_entries_mut::<PriceBuffer, i64>(price_buffer_acc.try_borrow_mut_data()?).unwrap();
    let mut time_series = load_entries_mut::<TimeSeries, i64>(time_series_acc.try_borrow_mut_data()?).unwrap();

    let mut next_instruction: Option<InstructionData> = None;

    match load_price_feed_from_account_info(&price_feed.to_account_info()) {
        Ok(price_feed) => { 
            // Load Pyth price fee. 
            let price = price_feed.get_price_unchecked();

            // Starting at the tail, start nullifying data points older than the lookback window.
            // TODO This is a worst-case linear operation over a large time_series. 
            //      Watch out for exceeding compute unit limits. Since this is a threaded instruction,
            //      we can run it as an infinite loop until we've cleared out all the old data.
            match stat.head {
                None => {}, // Noop
                Some(head) => {
                    let mut tail = (head - stat.sample_count as i64 + 1).rem_euclid(stat.buffer_size as i64);
                    while stat.sample_count > 0 && time_series[tail as usize] < price.publish_time - stat.lookback_window {
                        stat.sample_sum -= price_buffer[tail as usize];
                        stat.sample_count -= 1;
                        price_buffer[tail as usize] = i64::default();
                        avg_buffer[tail as usize] = i64::default();
                        time_series[tail as usize] = i64::default();
                        tail = (tail + 1).rem_euclid(stat.buffer_size as i64);
                    }
                }
            }
            
            // Insert the new data point, and update head. 
            match stat.head {
                // no data present
                None => {
                    stat.head = Some(0);
                    time_series[0] = price.publish_time;
                    price_buffer[0] = price.price;
                    stat.sample_count += 1;
                },
                // data present
                Some(head) => {
                    // update head idx for next insertion
                    stat.head = Some((head + 1).rem_euclid(stat.buffer_size as i64));

                    // If the buffer is not yet full, increment the sample count. 
                    // Otherwise, subtract the data value that's about to be overwritten from the sum.  
                    if stat.sample_count < stat.buffer_size {
                        stat.sample_count += 1
                    } else {
                        stat.sample_sum -= price_buffer[stat.head.unwrap() as usize];
                    }

                    // Insert the new data point.
                    price_buffer[stat.head.unwrap() as usize] = price.price;
                    time_series[stat.head.unwrap() as usize] = price.publish_time;
                }
            };

            // Update the sum and average
            stat.sample_sum += price.price;
            avg_buffer[stat.head.unwrap() as usize]  = stat.sample_sum.checked_div(stat.sample_count as i64).unwrap();

            // set next_instruction to realloc account size for zero copy data accounts
            let new_realloc_size: usize = 8 + std::mem::size_of::<TimeSeries>() + ((stat.buffer_size + 1280) * std::mem::size_of::<i64>());
            if stat.sample_count == stat.buffer_size && stat.head.unwrap() == (stat.buffer_size - 1) as i64 && new_realloc_size < 10_000_000 {
                next_instruction = 
                    Some(InstructionData { 
                            program_id: crate::ID, 
                            accounts: vec![
                                AccountMetaData::new(avg_buffer_acc.key(), false),
                                AccountMetaData::new(clockwork_sdk::utils::PAYER_PUBKEY, true),
                                AccountMetaData::new(price_buffer_acc.key(), false),
                                AccountMetaData::new(stat.key(), false),
                                AccountMetaData::new_readonly(system_program::ID, false),
                                AccountMetaData::new(thread.key(), true),
                                AccountMetaData::new(time_series_acc.key(), false),
                            ], 
                            data: clockwork_sdk::utils::anchor_sighash("realloc_buffers").to_vec() 
                        });
            }

            let tail = (stat.head.unwrap() - stat.sample_count as i64 + 1).rem_euclid(stat.buffer_size as i64);
            
            msg!("------------LIVE DATA------------");
            msg!("      live time: {}", price.publish_time);
            msg!("     live price: {}", price.price);
            msg!("--------STATS ACCOUNT DATA-------");
            msg!("     price feed: {}", stat.price_feed);
            msg!("      authority: {}", stat.authority);
            msg!("    oldest - ts: {}", time_series.get(tail as usize).unwrap());
            msg!("    newest - ts: {}", time_series.get(stat.head.unwrap() as usize).unwrap());
            msg!("      avg price: {}", avg_buffer.get(stat.head.unwrap() as usize).unwrap());
            msg!(" lookback window: {} seconds", stat.lookback_window);
            msg!("    sample rate: {:?}", thread.trigger);
            msg!("   sample count: {}", stat.sample_count);
            msg!("     sample sum: {}", stat.sample_sum);
            msg!("    buffer size: {}", stat.buffer_size);
            msg!("           head: {}", stat.head.unwrap());
            msg!("           tail: {}", tail);
            msg!("---------------------------------");
        },
        Err(_) => {},
    }

    Ok(ThreadResponse { next_instruction, ..ThreadResponse::default() })
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
