pub mod id;
pub mod state;

mod instructions;

pub use id::ID;

use anchor_lang::prelude::*;
use instructions::*;

#[program]
pub mod hello_clockwork {
    use super::*;

    pub fn create_queue<'info>(ctx: Context<'_, '_, '_, 'info, CreateQueue<'info>>) -> Result<()> {
        create_queue::handler(ctx)
    }

    pub fn create_task<'info>(ctx: Context<'_, '_, '_, 'info, CreateTask<'info>>) -> Result<()> {
        create_task::handler(ctx)
    }

    pub fn hello_world(
        ctx: Context<HelloWorld>,
    ) -> Result<clockwork_scheduler::response::TaskResponse> {
        hello_world::handler(ctx)
    }
}
