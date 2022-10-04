use crate::*;

#[derive(Accounts)]
pub struct Unsubscribe<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Unsubscribe<'_> {
    pub fn process(&mut self) -> Result<()> {
        let Self { payer, .. } = self;

        Ok(())
    }
}
