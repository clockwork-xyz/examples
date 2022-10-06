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
    ) -> Result<()> {
        ctx.accounts
            .process(recurrent_amount, epochs_reset, mint, is_active)
    }

    /*
     * subscribe to a subscription
     */
    pub fn subscribe<'info>(ctx: Context<Subscribe>) -> Result<()> {
        let bump = *ctx.bumps.get("payment").unwrap();
        ctx.accounts.process(bump)
    }

    /*
     * unsubscribe from a subscription
     */
    pub fn unsubscribe<'info>(ctx: Context<Unsubscribe>) -> Result<()> {
        ctx.accounts.process()
    }
}
