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

    pub fn initialize<'info>(ctx: Context<'_, '_, '_, 'info, Initialize<'info>>) -> Result<()> {
        initialize::handler(ctx)
    }

    pub fn deposit<'info>(
        ctx: Context<'_, '_, '_, 'info, Deposit<'info>>,
        amount: u64,
        transfer_rate: u64,
    ) -> Result<()> {
        deposit::handler(ctx, amount, transfer_rate)
    }

    pub fn auto_withdraw<'info>(
        ctx: Context<'_, '_, '_, 'info, AutoWithdraw<'info>>,
    ) -> Result<()> {
        auto_withdraw::handler(ctx)
    }

    pub fn disburse_payment(ctx: Context<'_, '_, '_, '_, DisbursePayment<'_>>) -> Result<()> {
        disburse_payment::handler(ctx)
    }
}
