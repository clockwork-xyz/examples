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
     * initialize clockworks related accounts
     */
    pub fn initialize<'info>(ctx: Context<'_, '_, '_, 'info, Initialize<'info>>) -> Result<()> {
        initialize::handler(ctx)
    }

    /*
     * delegate funds to manager (clockworks owned account) that will enable for automated swaps
     */
    pub fn delegate_funds<'info>(
        ctx: Context<'_, '_, '_, 'info, DelegateFunds<'info>>,
    ) -> Result<()> {
        delegate_funds::handler(ctx)
    }

    /*
     * makes cpi to serum dex to init open order account
     */
    pub fn init_oo_account<'info>(
        ctx: Context<'_, '_, '_, 'info, InitOOAccount<'info>>,
    ) -> Result<()> {
        init_oo_account::handler(ctx)
    }

    /*
     * clockworks automation for auto swapping on serum dex
     */
    pub fn auto_swap<'info>(ctx: Context<'_, '_, '_, 'info, AutoSwap<'info>>) -> Result<()> {
        auto_swap::handler(ctx)
    }

    /*
     * swap
     */
    pub fn swap<'info>(ctx: Context<'_, '_, '_, 'info, Swap<'info>>) -> Result<()> {
        swap::handler(ctx)
    }
}
