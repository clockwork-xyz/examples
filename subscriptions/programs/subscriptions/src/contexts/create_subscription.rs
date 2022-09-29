use crate::*;

#[derive(Accounts)]
pub struct CreateSubscription<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info,System>
}

impl<'info> CreateSubscription<'_> {
    pub fn process(&mut self,recurrent_amount:u64,epochs_reset:u64,start_schedule:String) -> Result<()> {
        let Self {
            payer,
            ..
        } = self;

        Ok(())
    }
}