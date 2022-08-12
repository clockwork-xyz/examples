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
     * makes cpi to serum dex to init open order account
     */
    pub fn create_orders<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateOrders<'info>>,
    ) -> Result<()> {
        create_orders::handler(ctx)
    }

    /*
     * create investments account and init queue/task accounts
     */
    pub fn create_investment<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateInvestment<'info>>,
        swap_amount: u64,
    ) -> Result<()> {
        create_investment::handler(ctx, swap_amount)
    }

    /*
     * deposit into investor account mint A token account
     */
    pub fn deposit<'info>(
        ctx: Context<'_, '_, '_, 'info, Deposit<'info>>,
        amount: u64,
    ) -> Result<()> {
        deposit::handler(ctx, amount)
    }

    /*
     * withdraw from investor account mint A token account
     */
    pub fn withdraw<'info>(
        ctx: Context<'_, '_, '_, 'info, Withdraw<'info>>,
        amount: u64,
    ) -> Result<()> {
        withdraw::handler(ctx, amount)
    }

    /*
     * withdraw from investor account mint B token account to payer's mint B token account
     */
    pub fn claim<'info>(ctx: Context<'_, '_, '_, 'info, Claim<'info>>, amount: u64) -> Result<()> {
        claim::handler(ctx, amount)
    }

    /*
     * swap ix
     */
    pub fn swap<'info>(ctx: Context<'_, '_, '_, 'info, Swap<'info>>) -> Result<()> {
        swap::handler(ctx)
    }
}
