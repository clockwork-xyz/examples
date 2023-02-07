use {crate::state::*, anchor_lang::prelude::*};

#[derive(Accounts)]
pub struct DcaDelete<'info> {
    /// The authority (owner) of the dca.
    #[account()]
    pub authority: Signer<'info>,

    /// The address to return the data rent lamports to.
    #[account(mut)]
    pub close_to: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [
            SEED_DCA,
            dca.authority.as_ref(),
            dca.market.as_ref(),
        ],
        bump,
        has_one = authority,
        close = close_to
    )]
    pub dca: Account<'info, Dca>,
}

pub fn handler(_ctx: Context<DcaDelete>) -> Result<()> {
    Ok(())
}
