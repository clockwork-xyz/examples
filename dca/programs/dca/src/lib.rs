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
     * create fund account
     */
    pub fn create_fund<'info>(ctx: Context<'_, '_, '_, 'info, CreateFund<'info>>) -> Result<()> {
        create_fund::handler(ctx)
    }

    /*
     * initiating swap
     */
    pub fn swap<'info>(ctx: Context<'_, '_, '_, 'info, Swap<'info>>) -> Result<()> {
        swap::handler(ctx)
    }

    /*
     * clockworks automation for auto swapping on serum dex
     */
    // pub fn auto_swap(ctx: Context<'_, '_, '_, 'info, AutoSwap<'info>>) -> Result<()> {
    // auto_swap::handler(ctx)
    // }
}
