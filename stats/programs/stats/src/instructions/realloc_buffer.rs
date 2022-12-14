use {
    crate::state::*,
    anchor_lang::{prelude::*,  solana_program::system_program, system_program::{Transfer, transfer}},
};

#[derive(Accounts)]
#[instruction(buffer_size: usize)]
pub struct ReallocBuffer<'info> {
    #[account(
        mut,
        seeds = [
            SEED_STAT, 
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

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<ReallocBuffer<'info>>, buffer_limit: usize) -> Result<()> {
    let payer = &ctx.accounts.payer;
    let dataset = &ctx.accounts.dataset;
    let stat = &mut ctx.accounts.stat;
    let system_program = &ctx.accounts.system_program;

    // Allocate more memory to stat account.
    let new_account_size = 8 + std::mem::size_of::<Dataset>() + (buffer_limit * std::mem::size_of::<crate::PriceData>());
    dataset.to_account_info().realloc(new_account_size, false)?;

    // Update the buffer limit.
    stat.buffer_limit = buffer_limit;

    // Transfer lamports to cover minimum rent requirements.
    let minimum_rent = Rent::get().unwrap().minimum_balance(new_account_size);
    if minimum_rent > dataset.to_account_info().lamports() {
        transfer(
            CpiContext::new(
                system_program.to_account_info(),
                Transfer {
                    from: payer.to_account_info(),
                    to: dataset.to_account_info(),
                },
            ),
            minimum_rent
                .checked_sub(dataset.to_account_info().lamports())
                .unwrap(),
        )?;
    }
    Ok(())
}

