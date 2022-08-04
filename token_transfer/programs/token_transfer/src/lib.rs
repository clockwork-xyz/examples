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

    pub fn create_queue<'info>(ctx: Context<'_, '_, '_, 'info, CreateQueue<'info>>) -> Result<()> {
        create_queue::handler(ctx)
    }

    pub fn create_escrow(
        ctx: Context<CreateEscrow>,
        amount: u64,
        transfer_rate: u64,
    ) -> Result<()> {
        create_escrow::handler(ctx, amount, transfer_rate)
    }

    pub fn deposit_funds<'info>(
        ctx: Context<'_, '_, '_, 'info, DepositFunds<'info>>,
    ) -> Result<()> {
        deposit_funds::handler(ctx)
    }

    pub fn create_task<'info>(ctx: Context<'_, '_, '_, 'info, CreateTask<'info>>) -> Result<()> {
        create_task::handler(ctx)
    }

    pub fn disburse_payment(ctx: Context<'_, '_, '_, '_, DisbursePayment<'_>>) -> Result<()> {
        disburse_payment::handler(ctx)
    }
}
