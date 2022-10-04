pub mod id;
pub mod state;

mod instructions;

pub use id::ID;

use anchor_lang::prelude::*;
use instructions::*;

#[program]
pub mod hello_clockwork {
    use super::*;

    pub fn initialize<'info>(ctx: Context<'_, '_, '_, 'info, Initialize<'info>>) -> Result<()> {
        initialize::handler(ctx)
    }

    pub fn hello_world(
        ctx: Context<HelloWorld>,
    ) -> Result<clockwork_sdk::queue_program::accounts::CrankResponse> {
        hello_world::handler(ctx)
    }
}
