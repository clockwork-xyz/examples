pub mod id;
pub mod state;

mod instructions;

pub use id::ID;

use anchor_lang::prelude::*;
use instructions::*;

#[program]
pub mod pyth_feed {
    use super::*;

    pub fn create_feed<'info>(ctx: Context<'_, '_, '_, 'info, CreateFeed<'info>>) -> Result<()> {
        create_feed::handler(ctx)
    }

    pub fn process_feed(ctx: Context<ProcessFeed>) -> Result<()> {
        process_feed::handler(ctx)
    }
}
