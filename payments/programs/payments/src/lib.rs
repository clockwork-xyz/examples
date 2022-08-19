pub mod id;
pub mod state;

mod instructions;

pub use id::ID;

use anchor_lang::prelude::*;
use instructions::*;

#[program]
pub mod payments_program {
    use super::*;

    pub fn create_payment<'info>(
        ctx: Context<'_, '_, '_, 'info, CreatePayment<'info>>,
        disbursement_amount: u64,
        schedule: String,
    ) -> Result<()> {
        create_payment::handler(ctx, disbursement_amount, schedule)
    }

    pub fn top_up_payment<'info>(
        ctx: Context<'_, '_, '_, 'info, TopUpPayment<'info>>,
        amount: u64,
    ) -> Result<()> {
        top_up_payment::handler(ctx, amount)
    }

    // TODO: Queue update interface not ready yet
    // pub fn update_payment<'info>(
    //     ctx: Context<'_, '_, '_, 'info, UpdatePayment<'info>>,
    //     disbursement_amount: Option<u64>,
    //     schedule: Option<String>,
    // ) -> Result<()> {
    //     update_payment::handler(ctx, disbursement_amount, schedule)
    // }

    pub fn disburse_payment<'info>(
        ctx: Context<'_, '_, '_, 'info, DisbursePayment<'info>>,
    ) -> Result<clockwork_crank::state::CrankResponse> {
        disburse_payment::handler(ctx)
    }
}
