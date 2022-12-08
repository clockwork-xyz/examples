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
        subscription_id: u64,
        bump: u8,
    ) -> Result<()> {
        ctx.accounts.process(
            recurrent_amount,
            schedule,
            mint,
            is_active,
            subscription_id,
            bump,
        )
    }

    /*
     * Deactivate subscription
     */
    pub fn deactivate_subscription<'info>(ctx: Context<DeactivateSubscription>) -> Result<()> {
        ctx.accounts.process()
    }

    /*
     * Withdraw from subscription bank
     */
    pub fn withdraw<'info>(ctx: Context<Withdraw>) -> Result<()> {
        ctx.accounts.process()
    }

    /*
     * Update Authority
     */
    pub fn update_authority<'info>(
        ctx: Context<UpdateAuthority>,
        new_authority: Pubkey,
    ) -> Result<()> {
        ctx.accounts.process(new_authority)
    }

    /*
     * create subscriber
     */
    pub fn create_subscriber<'info>(
        ctx: Context<CreateSubscriber>,
        subscriber_bump: u8,
    ) -> Result<()> {
        ctx.accounts.process(subscriber_bump)
    }

    /*
     * subscribe from a subscription
     */
    pub fn subscribe<'info>(ctx: Context<Subscribe>) -> Result<()> {
        ctx.accounts.process()
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
    ) -> Result<clockwork_sdk::ThreadResponse> {
        ctx.accounts.process()
    }
}
