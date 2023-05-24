use anchor_lang::prelude::*;

declare_id!("CvZJMZSFu66dxPh8B8ZiURpSSgX1AXs49KPkdgckzzGn");

#[program]
pub mod lookup_tables{
    use super::*;

    pub fn dump(ctx: Context<Dump>) -> Result<()> {

        let account_info_iter = &ctx.remaining_accounts.iter();

        msg!(
            "LEN: {:#?}",
            &account_info_iter.len(),
        );
        msg!(
            "remainings: {:#?}",
            &account_info_iter,
        );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Dump{}
