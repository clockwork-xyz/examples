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
            stat.load()?.price_feed.as_ref(), 
            stat.load()?.authority.as_ref(),
            &stat.load()?.lookback_window.to_le_bytes(),
        ],
        bump
    )]
    pub stat: AccountLoader<'info, Stat>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<ReallocBuffer<'info>>, buffer_limit: usize) -> Result<()> {
    let payer = &ctx.accounts.payer;
    let stat = &ctx.accounts.stat;
    let system_program = &ctx.accounts.system_program;

    // Allocate more memory to stat account.
    let new_account_size = 8 + std::mem::size_of::<Stat>() + (buffer_limit * std::mem::size_of::<crate::PriceData>());
    stat.to_account_info().realloc(new_account_size, false)?;

    // Update the buffer limit.
    let stat_mut = &mut stat.load_init()?;
    stat_mut.buffer_limit = buffer_limit;

    // Transfer lamports to cover minimum rent requirements.
    let minimum_rent = Rent::get().unwrap().minimum_balance(new_account_size);
    if minimum_rent > stat.to_account_info().lamports() {
        transfer(
            CpiContext::new(
                system_program.to_account_info(),
                Transfer {
                    from: payer.to_account_info(),
                    to: stat.to_account_info(),
                },
            ),
            minimum_rent
                .checked_sub(stat.to_account_info().lamports())
                .unwrap(),
        )?;
    }
    Ok(())
}

