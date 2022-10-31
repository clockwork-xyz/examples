pub mod id;
pub mod state;

mod instructions;

pub use id::ID;

use anchor_lang::prelude::*;
use instructions::*;

#[program]
pub mod serum_crank {
    use super::*;

    /*
     * initialize clockwork thread
     */
    pub fn initialize<'info>(ctx: Context<'_, '_, '_, 'info, Initialize<'info>>) -> Result<()> {
        initialize::handler(ctx)
    }

    /*
     * read events from event queue
     */
    pub fn read_events<'info>(
        ctx: Context<'_, '_, '_, 'info, ReadEvents<'info>>,
    ) -> Result<clockwork_sdk::ExecResponse> {
        read_events::handler(ctx)
    }

    /*
     * crank events event queue
     */
    pub fn consume_events<'info>(
        ctx: Context<'_, '_, '_, 'info, ConsumeEvents<'info>>,
    ) -> Result<clockwork_sdk::ExecResponse> {
        consume_events::handler(ctx)
    }
}
