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
            SEED_DATASET, 
            stat.key().as_ref(), 
        ],
        bump
    )]
    pub dataset: AccountLoader<'info, Dataset>,

    #[account(
        mut,
        seeds = [
            SEED_HISTORICAL_AVGS,
            stat.key().as_ref(),
        ],
        bump
    )]
    pub historical_avgs: AccountLoader<'info, HistoricalAvgs>,

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

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(
        constraint = thread.authority == stat.authority,
        address = thread.pubkey(),
        signer
    )]
    pub thread: Account<'info, Thread>,
}

pub fn handler<'info>(ctx: Context<ReallocBuffers<'info>>) -> Result<ThreadResponse> {
    let payer = &ctx.accounts.payer;
    let dataset = ctx.accounts.dataset.as_ref();
    let historical_avgs = ctx.accounts.historical_avgs.as_ref();
    let stat = &mut ctx.accounts.stat;
    let system_program = &ctx.accounts.system_program;
    let thread = &ctx.accounts.thread;

    // (1024 * 10) / 16 bytes = 640
    let new_buffer_size: usize = stat.buffer_size + 640;

    // Allocate more memory to account(s).
    let dataset_new_account_size = 8 + std::mem::size_of::<Dataset>() + (new_buffer_size * std::mem::size_of::<PriceData>());
    let historical_avgs_new_account_size = 8 + std::mem::size_of::<HistoricalAvgs>() + (new_buffer_size * std::mem::size_of::<i64>());

    dataset.realloc(dataset_new_account_size, false)?;
    historical_avgs.realloc(historical_avgs_new_account_size, false)?;

    // Update the buffer size.
    stat.buffer_size = new_buffer_size;

    // Transfer lamports to cover minimum rent requirements.
    let dataset_minimum_rent = Rent::get().unwrap().minimum_balance(dataset_new_account_size);
    let historical_avgs_minimum_rent = Rent::get().unwrap().minimum_balance(historical_avgs_new_account_size);

    if dataset_minimum_rent > dataset.to_account_info().lamports() {
        transfer(
            CpiContext::new(
                system_program.to_account_info(),
                Transfer {
                    from: payer.to_account_info(),
                    to: dataset.to_account_info(),
                },
            ),
            dataset_minimum_rent
                .checked_sub(dataset.to_account_info().lamports())
                .unwrap(),
        )?;
    }

    if historical_avgs_minimum_rent > historical_avgs.to_account_info().lamports() {
        transfer(
            CpiContext::new(
                system_program.to_account_info(),
                Transfer {
                    from: payer.to_account_info(),
                    to: historical_avgs.to_account_info(),
                },
            ),
            historical_avgs_minimum_rent
                .checked_sub(historical_avgs.to_account_info().lamports())
                .unwrap(),
        )?;
    }

    Ok(ThreadResponse { 
        next_instruction: Some(InstructionData {
            program_id: crate::ID,
            accounts: vec![
                AccountMetaData::new(dataset.key(), false),
                AccountMetaData::new(historical_avgs.key(), false),
                AccountMetaData::new(stat.key(), false),
                AccountMetaData::new_readonly(stat.price_feed, false),
                AccountMetaData::new(thread.key(), true),
            ],
            data: clockwork_sdk::utils::anchor_sighash("calc").to_vec()
        }),
        ..ThreadResponse::default() 
    })
}

