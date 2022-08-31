pub mod id;
pub mod state;

mod instructions;

pub use id::ID;

use anchor_lang::prelude::*;
use instructions::*;

#[program]
pub mod distributor {
    use super::*;

    /*
     * initialize clockwork queue
     */
    pub fn initialize<'info>(ctx: Context<'_, '_, '_, 'info, Initialize<'info>>) -> Result<()> {
        initialize::handler(ctx)
    }

    /*
     * Mint and distribute tokens to recipient
     */
    pub fn mint_token<'info>(
        ctx: Context<'_, '_, '_, 'info, MintToken<'info>>,
        mint_amount: u64,
    ) -> Result<clockwork_crank::state::CrankResponse> {
        mint_token::handler(ctx, mint_amount)
    }

    /*
     * update recipient and clockwork queue ix data
     */
    pub fn set_recipient<'info>(
        ctx: Context<'_, '_, '_, 'info, SetRecipient<'info>>,
        new_recipient: Option<Pubkey>,
    ) -> Result<()> {
        set_recipient::handler(ctx, new_recipient)
    }
}
