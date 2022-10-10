pub mod id;
pub mod state;

mod instructions;

pub use id::ID;

use anchor_lang::prelude::*;
use instructions::*;

#[program]
pub mod pyth_feed {
    use super::*;

    pub fn create_feed<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateFeed<'info>>,
        pyth_feed: Pubkey,
    ) -> Result<()> {
        create_feed::handler(ctx, pyth_feed)
    }

    pub fn process_pyth_feed(ctx: Context<ProcessPythFeed>) -> Result<()> {
        process_pyth_feed::handler(ctx)
    }
}
