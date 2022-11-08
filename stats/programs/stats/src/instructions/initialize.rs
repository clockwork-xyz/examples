use {
    crate::objects::*,
    anchor_lang::{prelude::*, solana_program::system_program},
    std::mem::size_of,
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds = [SEED_STAT, signer.key().as_ref()],
        bump,
        payer = signer,
        space = 8 + size_of::<Stat>(),
    )]
    pub stat: Account<'info, Stat>,

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<Initialize<'info>>, lookback_window: u64) -> Result<()> {
    let stat = &mut ctx.accounts.stat;

    stat.new(lookback_window)?;

    Ok(())
}
