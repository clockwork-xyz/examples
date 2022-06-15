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
     * amount: u64 - deposit from sender
     * transfer_rate: u64 - rate at which deposit is sent to receiver from escrow
     */

    pub fn create_payment<'info>(
        ctx: Context<'_, '_, '_, 'info, CreatePayment<'info>>,
        amount: u64,
        transfer_rate: u64,
    ) -> Result<()> {
        create_payment::handler(ctx, amount, transfer_rate)
    }

    pub fn initialize<'info>(ctx: Context<'_, '_, '_, 'info, Initialize<'info>>) -> Result<()> {
        initialize::handler(ctx)
    }

    pub fn disburse_payment(ctx: Context<'_, '_, '_, '_, DisbursePayment<'_>>) -> Result<()> {
        disburse_payment::handler(ctx)
    }
}
