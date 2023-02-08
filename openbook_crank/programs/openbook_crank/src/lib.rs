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
    pub fn initialize<'info>(ctx: Context<'_, '_, '_, 'info, Initialize<'info>>) -> Result<()> {
        initialize::handler(ctx)
    }

    /*
     * crank open orders
     */
    pub fn consume_events<'info>(
        ctx: Context<'_, '_, '_, 'info, ConsumeEvents<'info>>,
    ) -> Result<clockwork_sdk::state::ThreadResponse> {
        consume_events::handler(ctx)
    }

    /*
     * delete crank account
     */
    pub fn delete<'info>(ctx: Context<'_, '_, '_, 'info, Delete<'info>>) -> Result<()> {
        delete::handler(ctx)
    }
}
