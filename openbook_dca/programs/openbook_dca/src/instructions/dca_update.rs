use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::system_program},
};

#[derive(Accounts)]
#[instruction(swap_amount: u64)]
pub struct DcaUpdate<'info> {
    #[account(
        mut,
        seeds = [SEED_DCA, payer.key().as_ref(), market.key().as_ref()],
        bump,
        has_one = market
    )]
    pub dca: Account<'info, Dca>,

    /// CHECK:
    pub market: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<DcaUpdate<'info>>, swap_amount: u64) -> Result<()> {
    let dca = &mut ctx.accounts.dca;

    dca.swap_amount = swap_amount;

    Ok(())
}
