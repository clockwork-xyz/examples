use crate::*;

#[derive(Accounts)]
pub struct Subscribe<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info,System>
}

impl<'info> Subscribe<'_> {
    pub fn process(&mut self) -> Result<()> {
        let Self {
            payer,
            ..
        } = self;

        Ok(())
    }
}