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
     * initialize distributor account
     */
    pub fn create<'info>(
        ctx: Context<'_, '_, '_, 'info, Create<'info>>,
        mint_amount: u64,
    ) -> Result<()> {
        create::handler(ctx, mint_amount)
    }

    /*
     * mint to recipient's ATA
     */
    pub fn distribute<'info>(
        ctx: Context<'_, '_, '_, 'info, Distribute<'info>>,
    ) -> Result<clockwork_sdk::state::ThreadResponse> {
        distribute::handler(ctx)
    }

    /*
     * update recipient, mint amount, and thread schedule
     */
    pub fn update<'info>(
        ctx: Context<'_, '_, '_, 'info, Update<'info>>,
        new_recipient: Option<Pubkey>,
        mint_amount: Option<u64>,
        schedule: Option<String>,
    ) -> Result<()> {
        update::handler(ctx, new_recipient, mint_amount, schedule)
    }
}
