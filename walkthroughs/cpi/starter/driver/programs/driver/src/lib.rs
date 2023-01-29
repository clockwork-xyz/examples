use anchor_lang::prelude::*;
mod id;
use id::ID;

#[program]
pub mod driver {
    use super::*;

    pub fn create_thread(ctx: Context<CreateThread>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateThread {}
