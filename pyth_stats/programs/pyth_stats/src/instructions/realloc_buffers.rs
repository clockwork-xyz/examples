use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::system_program, system_program::{Transfer, transfer}},
    clockwork_sdk::state::{ ThreadResponse, InstructionData, AccountMetaData, Thread, ThreadAccount },
};

#[derive(Accounts)]
pub struct ReallocBuffers<'info> {
    #[account(
        mut,
        seeds = [
            SEED_AVG_BUFFER,
            stat.key().as_ref(),
        ],
        bump
    )]
    pub avg_buffer: AccountLoader<'info, AvgBuffer>,
    
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SEED_PRICE_BUFFER,
            stat.key().as_ref(),
        ],
        bump
    )]
    pub price_buffer: AccountLoader<'info, PriceBuffer>,

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

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

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

pub fn handler<'info>(ctx: Context<ReallocBuffers<'info>>) -> Result<ThreadResponse> {
    let avg_buffer = ctx.accounts.avg_buffer.as_ref();
    let payer = &ctx.accounts.payer;
    let price_buffer = ctx.accounts.price_buffer.as_ref();
    let stat = &mut ctx.accounts.stat;
    let system_program = &ctx.accounts.system_program;
    let thread = &ctx.accounts.thread;
    let time_series = ctx.accounts.time_series.as_ref();

    // (1024 * 10) / 8 bytes = 1280
    let new_buffer_size: usize = stat.buffer_size + 1280;
    
    // Update the buffer size.
    stat.buffer_size = new_buffer_size;

    // get new account sizes
    let avg_buffer_new_account_size = 8 + std::mem::size_of::<AvgBuffer>() + (stat.buffer_size * std::mem::size_of::<i64>());
    let price_buffer_new_account_size = 8 + std::mem::size_of::<PriceBuffer>() + (stat.buffer_size * std::mem::size_of::<i64>());
    let time_series_new_account_size = 8 + std::mem::size_of::<TimeSeries>() + (stat.buffer_size * std::mem::size_of::<i64>());

    // reallocate with new account sizes
    avg_buffer.realloc(avg_buffer_new_account_size, false)?;
    price_buffer.realloc(price_buffer_new_account_size, false)?;
    time_series.realloc(time_series_new_account_size, false)?;

    // get min rent exemption amount
    let avg_buffer_minimum_rent = Rent::get().unwrap().minimum_balance(avg_buffer_new_account_size);
    let price_buffer_minimum_rent = Rent::get().unwrap().minimum_balance(price_buffer_new_account_size);
    let time_series_minimum_rent = Rent::get().unwrap().minimum_balance(time_series_new_account_size);
    
    // Transfer lamports to cover minimum rent requirements.
    if avg_buffer_minimum_rent > avg_buffer.to_account_info().lamports() {
        transfer(
            CpiContext::new(
                system_program.to_account_info(),
                Transfer {
                    from: payer.to_account_info(),
                    to: avg_buffer.to_account_info(),
                },
            ),
            avg_buffer_minimum_rent
                .checked_sub(avg_buffer.to_account_info().lamports())
                .unwrap(),
        )?;
    }

    if price_buffer_minimum_rent > price_buffer.to_account_info().lamports() {
        transfer(
            CpiContext::new(
                system_program.to_account_info(),
                Transfer {
                    from: payer.to_account_info(),
                    to: price_buffer.to_account_info(),
                },
            ),
            price_buffer_minimum_rent
                .checked_sub(price_buffer.to_account_info().lamports())
                .unwrap(),
        )?;
    }

    if time_series_minimum_rent > time_series.to_account_info().lamports() {
        transfer(
            CpiContext::new(
                system_program.to_account_info(),
                Transfer {
                    from: payer.to_account_info(),
                    to: time_series.to_account_info(),
                },
            ),
            time_series_minimum_rent
                .checked_sub(time_series.to_account_info().lamports())
                .unwrap(),
        )?;
    }

    Ok(ThreadResponse { 
        next_instruction: Some(InstructionData {
            program_id: crate::ID,
            accounts: vec![
                AccountMetaData::new(avg_buffer.key(), false),
                AccountMetaData::new(price_buffer.key(), false),
                AccountMetaData::new_readonly(stat.price_feed, false),
                AccountMetaData::new(stat.key(), false),
                AccountMetaData::new(thread.key(), true),
                AccountMetaData::new(time_series.key(), false),
            ],
            data: clockwork_sdk::utils::anchor_sighash("calc").to_vec()
        }),
        ..ThreadResponse::default() 
    })
}

