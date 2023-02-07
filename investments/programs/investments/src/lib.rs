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
    pub fn investment_create<'info>(
        ctx: Context<'_, '_, '_, 'info, InvestmentCreate<'info>>,
        swap_amount: u64,
    ) -> Result<()> {
        investment_create::handler(ctx, swap_amount)
    }

    /*
     * update investment account's swap amount
     */
    pub fn investment_update<'info>(
        ctx: Context<InvestmentUpdate<'info>>,
        swap_amount: u64,
    ) -> Result<()> {
        investment_update::handler(ctx, swap_amount)
    }

    /*
     * delete investment account
     */
    pub fn investment_delete<'info>(ctx: Context<InvestmentDelete<'info>>) -> Result<()> {
        investment_delete::handler(ctx)
    }

    /*
     * place order on openbook dex
     */
    pub fn swap<'info>(ctx: Context<'_, '_, '_, 'info, Swap<'info>>) -> Result<()> {
        swap::handler(ctx)
    }
}
