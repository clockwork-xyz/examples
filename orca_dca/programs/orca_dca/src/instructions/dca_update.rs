use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::system_program},
};

#[derive(Accounts)]
#[instruction(amount_in: Option<u64>, minimum_amount_in: Option<u64>)]
pub struct DcaUpdate<'info> {
    #[account(
        mut,
        seeds = [
            SEED_DCA, 
            dca.authority.as_ref(), 
            dca.a_mint.as_ref(),
            dca.b_mint.as_ref()
        ],
        bump,
    )]
    pub dca: Account<'info, Dca>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<DcaUpdate<'info>>, amount_in: Option<u64>, minimum_amount_out: Option<u64>) -> Result<()> {
    let dca = &mut ctx.accounts.dca;

    if let Some(ai) = amount_in {
        dca.amount_in = ai;
    }

    if let Some(mao) = minimum_amount_out {
        dca.minimum_amount_out = mao;
    }

    Ok(())
}
