use {
    crate::state::*,
    anchor_lang::{prelude::*, Discriminator, solana_program::system_program},
    bytemuck::{Pod, Zeroable},
    clockwork_sdk::state::{Thread, ThreadAccount, ThreadResponse, InstructionData, AccountMetaData},
    pyth_sdk_solana::{load_price_feed_from_account_info, Price},
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
    let thread = &ctx.accounts.thread;
    let dataset = ctx.accounts.dataset.as_ref();
    let mut data_points = load_entries_mut::<Dataset, PriceData>(dataset.try_borrow_mut_data()?).unwrap();

    let mut kickoff_instruction: Option<InstructionData> = None;
    let mut next_instruction: Option<InstructionData> = None;

    match load_price_feed_from_account_info(&price_feed.to_account_info()) {
        Ok(price_feed) => { 
            // Load Pyth price fee. 
            let price = price_feed.get_price_unchecked();
            let price_data = PriceData::from(price);

            // Starting at the tail, start nullifying data points older than the lookback window.
            // TODO This is a worst-case linear operation over a large dataset. 
            //      Watch out for exceeding compute unit limits. Since this is a threaded instruction,
            //      we can run it as an infinite loop until we've cleared out all the old data.
            match stat.head {
                None => {}, // Noop
                Some(head) => {
                    let mut tail = (head - stat.sample_count as i64 + 1).rem_euclid(stat.buffer_size as i64);
                    while stat.sample_count > 0 && data_points[tail as usize].ts < price.publish_time - stat.lookback_window {
                        stat.sample_sum -= data_points[tail as usize].price;
                        stat.sample_count -= 1;
                        data_points[tail as usize] = PriceData::default();
                        tail = (tail + 1).rem_euclid(stat.buffer_size as i64);
                    }
                }
            }
            
            // Insert the new data point, and update head. 
            match stat.head {
                // no data present
                None => {
                    stat.head = Some(0);
                    data_points[0] = price_data;
                    stat.sample_count += 1;
                },
                // data present
                Some(head) => {
                    // Exit early if this price is too early for the sampling rate.
                    if price.publish_time < data_points[head as usize].ts + stat.sample_rate {
                        return Ok(ThreadResponse::default())
                    }

                    // update head idx for next insertion
                    stat.head = Some((head + 1).rem_euclid(stat.buffer_size as i64));

                    // If the buffer is not yet full, increment the sample count. 
                    // Otherwise, subtract the data value that's about to be overwritten from the sum.  
                    if stat.sample_count < stat.buffer_size {
                        stat.sample_count += 1
                    } else {
                        stat.sample_sum -= data_points[stat.head.unwrap() as usize].price;
                    }

                    // Insert the new data point.
                    data_points[stat.head.unwrap() as usize] = price_data;
                }
            };

            // Update the sum and average
            stat.sample_sum += price_data.price;
            stat.sample_avg  = stat.sample_sum.checked_div(stat.sample_count as i64).unwrap();

            let new_realloc_size: usize = 8 + std::mem::size_of::<Dataset>() + ((stat.buffer_size + 640) * std::mem::size_of::<crate::PriceData>());
            if stat.sample_count == stat.buffer_size && stat.head.unwrap() == (stat.buffer_size - 1) as i64 && new_realloc_size < 10_000_000 {
                next_instruction = 
                    Some(InstructionData { 
                            program_id: crate::ID, 
                            accounts: vec![
                                AccountMetaData::new(dataset.key(), false),
                                AccountMetaData::new(stat.key(), false),
                                AccountMetaData::new(clockwork_sdk::utils::PAYER_PUBKEY, true),
                                AccountMetaData::new_readonly(system_program::ID, false),
                                AccountMetaData::new(thread.key(), true),
                            ], 
                            data: clockwork_sdk::utils::anchor_sighash("realloc_buffer").to_vec() 
                        });
            } else {
                kickoff_instruction = Some(InstructionData {
                    program_id: crate::ID,
                    accounts: vec![
                        AccountMetaData::new(dataset.key(), false),
                        AccountMetaData::new(stat.key(), false),
                        AccountMetaData::new_readonly(stat.price_feed, false),
                        AccountMetaData::new(thread.key(), true),
                    ],
                    data: clockwork_sdk::utils::anchor_sighash("calc").to_vec()
                }) 
            }

            let tail = (stat.head.unwrap() - stat.sample_count as i64 + 1).rem_euclid(stat.buffer_size as i64);
            
            msg!("------------LIVE DATA------------");
            msg!("     live price: {}", price_data.price);
            msg!("      live time: {}", price_data.ts);
            msg!("--------STATS ACCOUNT DATA-------");
            msg!("     price feed: {}", stat.price_feed);
            msg!("      authority: {}", stat.authority);
            msg!("    oldest - ts: {}", data_points.get(tail as usize).unwrap().ts);
            msg!("    newest - ts: {}", data_points.get(stat.head.unwrap() as usize).unwrap().ts);
            msg!("      avg price: {}", stat.sample_avg);
            msg!("lookback window: {} seconds", stat.lookback_window);
            msg!("    sample rate: {}", stat.sample_rate);
            msg!("   sample count: {}", stat.sample_count);
            msg!("     sample sum: {}", stat.sample_sum);
            msg!("    buffer_size: {}", stat.buffer_size);
            msg!("           head: {}", stat.head.unwrap());
            msg!("           tail: {}", tail);
            msg!("---------------------------------");
        },
        Err(_) => {},
    }

    Ok(ThreadResponse { kickoff_instruction, next_instruction })
}

#[derive(Copy, Clone, Zeroable, Pod, Default)]
#[repr(C)]
pub struct PriceData {
    pub price: i64,
    pub ts: i64,
}

impl From<Price> for PriceData {
    fn from(price: Price) -> Self {
        PriceData { price: price.price, ts: price.publish_time }
    }
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
