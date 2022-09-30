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
    pub fn initialize<'info>(
        ctx: Context<'_, '_, '_, 'info, Initialize<'info>>,
        mint_amount: u64,
    ) -> Result<()> {
        initialize::handler(ctx, mint_amount)
    }

    /*
     * mint to recipient's ATA
     */
    pub fn mint_token<'info>(
        ctx: Context<'_, '_, '_, 'info, MintToken<'info>>,
    ) -> Result<clockwork_sdk::queue_program::state::CrankResponse> {
        mint_token::handler(ctx)
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
