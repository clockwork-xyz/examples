use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::system_program},
};

#[derive(Accounts)]
#[instruction(settings: DcaSettings)]
pub struct DcaUpdate<'info> {
    #[account(
        mut,
        seeds = [
            SEED_DCA, 
            dca.authority.as_ref(), 
            dca.a_mint.as_ref(), 
            dca.b_mint.as_ref(), 
        ],
        bump,
    )]
    pub dca: Account<'info, Dca>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<DcaUpdate<'info>>, settings: DcaSettings) -> Result<()> {
    let dca = &mut ctx.accounts.dca;

    if let Some(a) = settings.amount {
        dca.amount = a;
    }

    if let Some(oat) = settings.other_amount_threshold {
        dca.other_amount_threshold = oat;
    }

    if let Some(spl) = settings.sqrt_price_limit {
        dca.sqrt_price_limit = spl;
    }

    if let Some(asii) = settings.amount_specified_is_input {
        dca.amount_specified_is_input = asii;
    }

    if let Some(a2b) = settings.a_to_b {
        dca.a_to_b = a2b;
    }

    Ok(())
}
