pub mod id;
pub mod pda;
pub mod state;

mod instructions;

pub use id::ID;

use anchor_lang::prelude::*;
use instructions::*;

#[program]
pub mod dca {
    use super::*;

    /*
     * create clockwork queue
     */
    pub fn create_queue<'info>(ctx: Context<'_, '_, '_, 'info, CreateQueue<'info>>) -> Result<()> {
        create_queue::handler(ctx)
    }

    /*
     * delegate funds to Authority (DCA owned account) that will enable for automated swaps
     */
    pub fn delegate_funds<'info>(
        ctx: Context<'_, '_, '_, 'info, DelegateFunds<'info>>,
    ) -> Result<()> {
        delegate_funds::handler(ctx)
    }

    /*
     * makes cpi to serum dex to init open order account
     */
    pub fn create_orders<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateOrders<'info>>,
    ) -> Result<()> {
        create_orders::handler(ctx)
    }

    /*
     * create clockwork task to then automate swapping on serum dex
     */
    pub fn create_task<'info>(ctx: Context<'_, '_, '_, 'info, CreateTask<'info>>) -> Result<()> {
        create_task::handler(ctx)
    }

    /*
     * swap ix
     */
    pub fn swap<'info>(ctx: Context<'_, '_, '_, 'info, Swap<'info>>) -> Result<()> {
        swap::handler(ctx)
    }
}
