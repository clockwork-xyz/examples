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
        schedule: String,
        mint: Pubkey,
        is_active: bool,
        subscription_id: String,
    ) -> Result<()> {
        ctx.accounts
            .process(recurrent_amount, schedule, mint, is_active, subscription_id)
    }

    /*
     * subscribe to a subscription
     */
    pub fn subscribe<'info>(ctx: Context<Subscribe>, subscription_period: u8) -> Result<()> {
        let bump = *ctx.bumps.get("subscription").unwrap();
        ctx.accounts.process(bump, subscription_period)
    }

    /*
     * unsubscribe from a subscription
     */
    pub fn unsubscribe<'info>(ctx: Context<Unsubscribe>) -> Result<()> {
        let bump = *ctx.bumps.get("subscription").unwrap();
        ctx.accounts.process(bump)
    }

    /*
     * disburse payment from program authority's ATA to recipient's ATA
     */
    pub fn disburse_payment<'info>(
        ctx: Context<DisbursePayment>,
    ) -> Result<clockwork_sdk::CrankResponse> {
        ctx.accounts.process()
    }
}
