use {crate::objects::*, anchor_lang::prelude::*};

#[derive(Accounts)]
pub struct Stats<'info> {}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, Stats<'info>>) -> Result<()> {
    Ok(())
}
