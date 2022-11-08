pub mod id;
pub mod objects;

mod instructions;

pub use id::ID;

use anchor_lang::prelude::*;
use instructions::*;

#[program]
pub mod stats {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, lookback_window: u64) -> Result<()> {
        initialize::handler(ctx, lookback_window)
    }
}
