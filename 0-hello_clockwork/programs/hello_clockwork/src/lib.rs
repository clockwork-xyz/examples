use anchor_lang::prelude::*;
pub mod id;
pub use id::ID;

#[program]
pub mod hello_clockwork {
    use super::*;

    pub fn hello(_ctx: Context<Hello>, name: String) -> Result<()> {
        msg!(
            "Hello, {}! The current time is: {}",
            name,
            Clock::get().unwrap().unix_timestamp
        );

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(name: String)]
pub struct Hello {}
// Replace the above by this to enforce that the ix can only be run from a given thread
// #[derive(Accounts)]
// pub struct Hello<'info> {
//     #[account(address = thread.pubkey(), signer)]
//     pub thread: Account<'info, Thread>,
// }
