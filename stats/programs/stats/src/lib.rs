#![feature(toowned_clone_into)]
pub mod id;
pub mod state;

mod instructions;

pub use id::ID;

use anchor_lang::prelude::*;
use instructions::*;

#[program]
pub mod stats {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        lookback_window: i64,
        sample_rate: i64,
    ) -> Result<()> {
        initialize::handler(ctx, lookback_window, sample_rate)
    }
    pub fn calc(ctx: Context<Calc>) -> Result<()> {
        calc::handler(ctx)
    }
}
