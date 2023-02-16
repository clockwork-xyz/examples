pub mod id;
pub mod state;

mod instructions;

pub use id::ID;

use anchor_lang::prelude::*;
use instructions::*;

#[program]
pub mod orca_whirlpool_dca {
    use super::*;

    /*
     * initialize dca account with orca swap params
     */
    pub fn dca_create<'info>(
        ctx: Context<'_, '_, '_, 'info, DcaCreate<'info>>,
        amount: u64,
        other_amount_threshold: u64,
        sqrt_price_limit: u128,
        amount_specified_is_input: bool,
        a_to_b: bool,
    ) -> Result<()> {
        dca_create::handler(
            ctx,
            amount,
            other_amount_threshold,
            sqrt_price_limit,
            amount_specified_is_input,
            a_to_b,
        )
    }

    /*
     * update dca settings
     */
    pub fn dca_update<'info>(
        ctx: Context<'_, '_, '_, 'info, DcaUpdate<'info>>,
        settings: crate::state::DcaSettings,
    ) -> Result<()> {
        dca_update::handler(ctx, settings)
    }

    /*
     * delete dca account
     */
    pub fn dca_delete<'info>(ctx: Context<'_, '_, '_, 'info, DcaDelete<'info>>) -> Result<()> {
        dca_delete::handler(ctx)
    }

    /*
     * get tick arrays for upcoming swap
     */
    pub fn get_tick_arrays<'info>(
        ctx: Context<GetTickArrays<'info>>,
    ) -> Result<clockwork_sdk::state::ThreadResponse> {
        get_tick_arrays::handler(ctx)
    }

    /*
     * swap on orca whirlpool
     */
    pub fn swap<'info>(ctx: Context<'_, '_, '_, 'info, Swap<'info>>) -> Result<()> {
        swap::handler(ctx)
    }
}
