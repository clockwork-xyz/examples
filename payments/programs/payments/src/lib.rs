pub mod id;
pub mod state;

mod instructions;

pub use id::ID;

use anchor_lang::prelude::*;
use instructions::*;

#[program]
pub mod payments_program {
    use super::*;

    /*
     * initialize relevant accounts and clockwork queue for automated payment flow
     */
    pub fn create_payment<'info>(
        ctx: Context<'_, '_, '_, 'info, CreatePayment<'info>>,
        disbursement_amount: u64,
        schedule: String,
    ) -> Result<()> {
        create_payment::handler(ctx, disbursement_amount, schedule)
    }

    /*
     * disburse payment from program authority's ATA to recipient's ATA
     */
    pub fn disburse_payment<'info>(
        ctx: Context<'_, '_, '_, 'info, DisbursePayment<'info>>,
    ) -> Result<clockwork_sdk::queue_program::state::CrankResponse> {
        disburse_payment::handler(ctx)
    }

    /*
     * deposit into program authority's ATA
     */
    pub fn top_up_payment<'info>(
        ctx: Context<'_, '_, '_, 'info, TopUpPayment<'info>>,
        amount: u64,
    ) -> Result<()> {
        top_up_payment::handler(ctx, amount)
    }

    /*
     * update disbursement amount and/or schedule
     */
    pub fn update_payment<'info>(
        ctx: Context<'_, '_, '_, 'info, UpdatePayment<'info>>,
        disbursement_amount: Option<u64>,
        schedule: Option<clockwork_sdk::queue_program::state::Trigger>,
    ) -> Result<()> {
        update_payment::handler(ctx, disbursement_amount, schedule)
    }
}
