pub mod id;
pub mod state;

mod instructions;

pub use id::ID;

use anchor_lang::prelude::*;
use instructions::*;

#[program]
pub mod payments {
    use super::*;

    /*
     * initialize relevant accounts and clockwork queue for automated payment flow
     */
    pub fn create_payment<'info>(
        ctx: Context<'_, '_, '_, 'info, CreatePayment<'info>>,
        amount: u64,
    ) -> Result<()> {
        create_payment::handler(ctx, amount)
    }

    /*
     * disburse payment from program authority's ATA to recipient's ATA
     */
    pub fn disburse_payment<'info>(
        ctx: Context<'_, '_, '_, '_, DisbursePayment<'_>>,
    ) -> Result<clockwork_sdk::CrankResponse> {
        disburse_payment::handler(ctx)
    }

    /*
     * update disbursement amount
     */
    pub fn update_payment<'info>(
        ctx: Context<'_, '_, '_, 'info, UpdatePayment<'info>>,
        amount: Option<u64>,
    ) -> Result<()> {
        update_payment::handler(ctx, amount)
    }
}
