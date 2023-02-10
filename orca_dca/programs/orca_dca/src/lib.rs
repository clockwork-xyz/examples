pub mod id;
pub mod state;

mod instructions;

pub use id::ID;

use anchor_lang::prelude::*;
use instructions::*;

#[program]
pub mod orca_dca {
    use super::*;

    /*
     * initialize dca account with swap params
     */
    pub fn dca_create<'info>(
        ctx: Context<'_, '_, '_, 'info, DcaCreate<'info>>,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Result<()> {
        dca_create::handler(ctx, amount_in, minimum_amount_out)
    }

    /*
     * update swap amount
     */
    pub fn dca_update<'info>(
        ctx: Context<'_, '_, '_, 'info, DcaUpdate<'info>>,
        amount_in: Option<u64>,
        minimum_amount_out: Option<u64>,
    ) -> Result<()> {
        dca_update::handler(ctx, amount_in, minimum_amount_out)
    }

    /*
     * delete dca account
     */
    pub fn dca_delete<'info>(ctx: Context<'_, '_, '_, 'info, DcaDelete<'info>>) -> Result<()> {
        dca_delete::handler(ctx)
    }

    /*
     * swap on orca whirlpool
     */
    pub fn proxy_swap<'info>(ctx: Context<'_, '_, '_, 'info, ProxySwap<'info>>) -> Result<()> {
        proxy_swap::handler(ctx)
    }
}
