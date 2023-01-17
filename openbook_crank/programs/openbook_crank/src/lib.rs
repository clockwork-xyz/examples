pub mod id;
pub mod state;

mod instructions;

pub use id::ID;

use anchor_lang::prelude::*;
use instructions::*;

#[program]
pub mod openbook_crank {
    use super::*;

    /*
     * initialize crank account
     */
    pub fn initialize<'info>(
        ctx: Context<'_, '_, '_, 'info, Initialize<'info>>,
        id: String,
    ) -> Result<()> {
        initialize::handler(ctx, id)
    }

    /*
     * read events from event queue
     */
    pub fn read_events<'info>(
        ctx: Context<'_, '_, '_, 'info, ReadEvents<'info>>,
    ) -> Result<clockwork_sdk::state::ThreadResponse> {
        read_events::handler(ctx)
    }

    /*
     * crank open orders
     */
    pub fn consume_events<'info>(
        ctx: Context<'_, '_, '_, 'info, ConsumeEvents<'info>>,
    ) -> Result<clockwork_sdk::state::ThreadResponse> {
        consume_events::handler(ctx)
    }
}
