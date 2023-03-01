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
     * Create Payment Approval
     */
    pub fn create_payment(ctx: Context<CreatePayment>, amount: u64) -> Result<()> {
        create_payment::handler(ctx, amount)
    }

    /*
     * disburse payment from program authority's ATA to recipient's ATA
     */
    pub fn disburse_payment(ctx: Context<DisbursePayment>) ->
    Result<clockwork_sdk::state::ThreadResponse> {
        disburse_payment::handler(ctx)
    }

    /*
     * update disbursement amount
     */
    pub fn update_payment(ctx: Context<UpdatePayment>, amount: Option<u64>) -> Result<()> {
        update_payment::handler(ctx, amount)
    }
}
