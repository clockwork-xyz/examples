pub mod id;
pub mod state;

mod instructions;

pub use id::ID;

use anchor_lang::prelude::*;
use instructions::*;

#[program]
pub mod openbook_dca {
    use super::*;

    /*
     * initialize dca and open orders accounts
     */
    pub fn dca_create<'info>(
        ctx: Context<'_, '_, '_, 'info, DcaCreate<'info>>,
        swap_amount: u64,
    ) -> Result<()> {
        dca_create::handler(ctx, swap_amount)
    }

    /*
     * update dca account's swap amount
     */
    pub fn dca_update<'info>(ctx: Context<DcaUpdate<'info>>, swap_amount: u64) -> Result<()> {
        dca_update::handler(ctx, swap_amount)
    }

    /*
     * delete dca account
     */
    pub fn dca_delete<'info>(ctx: Context<DcaDelete<'info>>) -> Result<()> {
        dca_delete::handler(ctx)
    }

    /*
     * transfer pc swap_amount from authority to dca
     * place order on openbook dex
     * settle funds
     * transfer coin token balance from dca to authority
     */
    pub fn swap<'info>(ctx: Context<'_, '_, '_, 'info, Swap<'info>>) -> Result<()> {
        swap::handler(ctx)
    }
}
