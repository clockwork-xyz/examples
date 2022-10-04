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
        start_schedule: String,
    ) -> Result<()> {
        ctx.accounts
            .process(recurrent_amount, epochs_reset, start_schedule)
    }

    /*
     * subscribe to a subscription
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
}
