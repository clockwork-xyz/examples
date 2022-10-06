pub mod id;
pub mod state;

mod instructions;

use anchor_lang::prelude::*;
use instructions::*;

pub use id::ID;

#[program]
pub mod auto_claim {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }
}
