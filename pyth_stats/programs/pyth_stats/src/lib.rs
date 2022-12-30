pub mod id;
pub mod state;

mod instructions;

pub use id::ID;

use anchor_lang::prelude::*;
use clockwork_sdk::state::*;
use instructions::*;

#[program]
pub mod stats {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, lookback_window: i64) -> Result<()> {
        initialize::handler(ctx, lookback_window)
    }
    pub fn calc(ctx: Context<Calc>) -> Result<ThreadResponse> {
        calc::handler(ctx)
    }

    pub fn realloc_buffers(ctx: Context<ReallocBuffers>) -> Result<ThreadResponse> {
        realloc_buffers::handler(ctx)
    }
}
