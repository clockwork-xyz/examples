pub mod id;

mod instructions;

pub use id::ID;

use anchor_lang::prelude::*;
use instructions::*;

#[program]
pub mod hello_clockwork {
    use super::*;

    pub fn hello_world(
        ctx: Context<HelloWorld>,
        name: String,
    ) -> Result<clockwork_sdk::state::ThreadResponse> {
        hello_world::handler(ctx, name)
    }
}
