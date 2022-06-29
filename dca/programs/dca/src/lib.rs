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
     * add documentation here
     */
    pub fn initialize<'info>(ctx: Context<'_, '_, '_, 'info, Initialize<'info>>) -> Result<()> {
        initialize::handler(ctx)
    }

    /*
     * add documentation here
     */
    pub fn create_fund(
        ctx: Context<'_, '_, '_, 'info, CreateFund<'info>>,
        name: String,
        symbol: String,
        assets: [Pubkey; 3],
        weights: [u64; 3],
        _token_decimals: u8,
    ) -> Result<()> {
        create_fund::handler(ctx, name, symbol, assets, weights, _token_decimals)
    }

    /*
     * add documentation here
     */
    pub fn init_order(ctx: Context<InitOrder>, amount: u64) -> Result<()> {
        init_order::handler(ctx, amount)
    }

    /*
     * add documentation here
     */
    pub fn fund_ata(ctx: Context<'_, '_, '_, 'info, FundAta<'info>>) -> Result<()> {
        fund_ata::handler(ctx)
    }

    /*
     * add documentation here
     */
    // pub fn auto_swap(ctx: Context<'_, '_, '_, 'info, AutoSwap<'info>>) -> Result<()> {
    // auto_swap::handler(ctx)
    // }

    /*
     * add documentation here
     */
    // pub fn swap(ctx: Context<'_, '_, '_, '_, Swap<'_>>) -> Result<()> {
    //     swap::handler(ctx)
    // }
}
