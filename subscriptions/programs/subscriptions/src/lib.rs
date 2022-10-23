pub mod contexts;
pub mod error;
pub mod id;
pub mod state;

use anchor_lang::prelude::*;
pub use contexts::*;
pub use id::ID;

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
     * create subscription queue
     */
    pub fn create_queue<'info>(ctx: Context<CreateQueue>) -> Result<()> {
        let bump = *ctx.bumps.get("subscription").unwrap();
        ctx.accounts.process(bump)
    }

    /*
     * create subscriber
     */
    pub fn create_subscriber<'info>(ctx: Context<CreateSubscriber>) -> Result<()> {
        ctx.accounts.process()
    }

    /*
     * deppsit
     */
    pub fn deposit<'info>(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.process(amount)
    }

    /*
     * withdraw
     */
    pub fn withdraw<'info>(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.process(amount)
    }

    /*
     * subscribe from a subscription
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
    ) -> Result<clockwork_sdk::CrankResponse> {
        let bump = *ctx.bumps.get("subscription").unwrap();
        ctx.accounts.process(bump)
    }
}
