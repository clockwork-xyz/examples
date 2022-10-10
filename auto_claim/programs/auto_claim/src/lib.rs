pub mod id;
pub mod state;

mod instructions;

use anchor_lang::prelude::*;
use instructions::*;

pub use id::ID;

#[program]
pub mod auto_claim {
    use super::*;

    /*
     * create a vesting contract and initialize queue to auto claim
     */
    pub fn create<'info>(
        ctx: Context<'_, '_, '_, 'info, Create<'info>>,
        schedule: String,
    ) -> Result<()> {
        create::handler(ctx, schedule)
    }

    /*
     * auto claim ix that gets invoked by queue
     */
    // pub fn auto_claim<'info>(ctx: Context<'_, '_, '_, 'info, AutoClaim<'info>>) -> Result<()> {
    //     auto_claim::handler(ctx)
    // }
}
