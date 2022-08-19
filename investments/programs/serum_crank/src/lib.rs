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
     * Initialize Queue with crank ix and schedule
     */
    pub fn initialize<'info>(ctx: Context<'_, '_, '_, 'info, Initialize<'info>>) -> Result<()> {
        initialize::handler(ctx)
    }

    /*
     * Crank events that are propogated to the event queue
     */
    pub fn crank_events<'info>(ctx: Context<'_, '_, '_, 'info, CrankEvents<'info>>) -> Result<()> {
        crank_events::handler(ctx)
    }
}
