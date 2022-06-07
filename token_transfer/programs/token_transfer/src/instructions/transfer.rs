use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::sysvar},
    cronos_scheduler::state::Manager,
};

#[derive(Accounts)]
pub struct Transfer {}

pub fn handler(ctx: Context<Transfer>) -> Result<()> {
    Ok(())
}
