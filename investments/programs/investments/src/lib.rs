pub mod id;
pub mod state;

mod instructions;

pub use id::ID;

use anchor_lang::prelude::*;
use instructions::*;

#[program]
pub mod investments_program {
    use super::*;

    /*
     * initialize investment and open orders accounts
     */
    pub fn create_investment<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateInvestment<'info>>,
        swap_amount: u64,
    ) -> Result<()> {
        create_investment::handler(ctx, swap_amount)
    }

    /*
     * swap
     */
    pub fn swap<'info>(ctx: Context<'_, '_, '_, 'info, Swap<'info>>) -> Result<()> {
        swap::handler(ctx)
    }

    /*
     * update investment account's swap amount
     */
    pub fn update_investment<'info>(
        ctx: Context<UpdateInvestment<'info>>,
        swap_amount: u64,
    ) -> Result<()> {
        update_investment::handler(ctx, swap_amount)
    }
}
