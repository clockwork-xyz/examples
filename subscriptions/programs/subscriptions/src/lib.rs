pub mod id;
pub mod state;

mod contexts;
use contexts::*;

pub use id::ID;

use anchor_lang::prelude::*;

#[program]
pub mod subscriptions_program {
    use super::*;

    /*
     * Initialize subscription account
     */
    pub fn create_subscription<'info>(
        ctx: Context<CreateSubscription>,
        recurrent_amount: u64,
        epochs_reset: u64,
        mint: Pubkey,
        is_active: bool,
        market_id: String,
    ) -> Result<()> {
        ctx.accounts
            .process(recurrent_amount, epochs_reset, mint, is_active, market_id)
    }

    /*
     * subscribe to a subscription
     */
    pub fn subscribe<'info>(ctx: Context<Subscribe>) -> Result<()> {
        let bump = *ctx.bumps.get("subscription").unwrap();
        ctx.accounts.process(bump)
    }

    /*
     * unsubscribe from a subscription
     */
    pub fn unsubscribe<'info>(ctx: Context<Unsubscribe>) -> Result<()> {
        ctx.accounts.process()
    }

    /*
     * disburse payment from program authority's ATA to recipient's ATA
     */
    pub fn disburse_payment<'info>(
        ctx: Context<DisbursePayment>,
    ) -> Result<clockwork_crank::state::CrankResponse> {
        let bump = *ctx.bumps.get("subscription").unwrap();
        ctx.accounts.process(bump)
    }
}
