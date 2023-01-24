use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::system_program},
};

#[derive(Accounts)]
#[instruction(swap_amount: u64)]
pub struct InvestmentUpdate<'info> {
    #[account(
        mut,
        seeds = [SEED_INVESTMENT, payer.key().as_ref(), market.key().as_ref()],
        bump,
        has_one = market
    )]
    pub investment: Account<'info, Investment>,

    /// CHECK:
    pub market: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<InvestmentUpdate<'info>>, swap_amount: u64) -> Result<()> {
    let investment = &mut ctx.accounts.investment;

    investment.swap_amount = swap_amount;

    Ok(())
}
