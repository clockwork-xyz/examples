pub mod id;

mod instructions;

pub use id::ID;

use anchor_lang::prelude::*;
use instructions::*;

#[program]
pub mod swap {
    use super::*;

    // grab tick arrays for orca whirlpool swap
    pub fn orca_whirlpool_preswap<'info>(
        ctx: Context<OrcaWhirlpoolPreSwap<'info>>,
        amount: u64,
        a_to_b: bool,
    ) -> Result<clockwork_sdk::state::ThreadResponse> {
        orca_whirlpool_preswap::handler(ctx, amount, a_to_b)
    }

    // perform orca whirlpool swap
    pub fn orca_whirlpool_swap<'info>(
        ctx: Context<'_, '_, '_, 'info, OrcaWhirlpoolSwap<'info>>,
        amount: u64,
        a_to_b: bool,
    ) -> Result<clockwork_sdk::state::ThreadResponse> {
        orca_whirlpool_swap::handler(ctx, amount, a_to_b)
    }
}
