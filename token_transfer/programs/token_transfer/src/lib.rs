pub mod errors;
pub mod id;
pub mod pda;
pub mod state;

mod instructions;

pub use id::ID;

use anchor_lang::prelude::*;
use instructions::*;

#[program]
pub mod token_transfer {
    use super::*;

    /**
     * deposit_amount: u64 - initial deposit from sender
     * transfer_rate: u64 - rate at which deposit is sent to receiver from escrow
     */

    pub fn initialize<'info>(
        ctx: Context<'_, '_, '_, 'info, Initialize<'info>>,
        deposit_amount: u64,
        transfer_rate: u64,
    ) -> Result<()> {
        initialize::handler(ctx, deposit_amount, transfer_rate)
    }

    pub fn transfer(ctx: Context<Transfer>) -> Result<()> {
        transfer::handler(ctx)
    }
}
