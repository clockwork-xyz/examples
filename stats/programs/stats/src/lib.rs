pub mod id;
pub mod objects;

mod instructions;

pub use id::ID;

use anchor_lang::prelude::*;
use instructions::*;

#[program]
pub mod stats {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, lookback_window: i64, id: String) -> Result<()> {
        initialize::handler(ctx, lookback_window, id)
    }
    pub fn calc(ctx: Context<Calc>) -> Result<()> {
        calc::handler(ctx)
    }
}
