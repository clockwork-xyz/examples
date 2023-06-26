pub mod id;
pub mod state;

mod instructions;

use anchor_lang::prelude::*;
use instructions::*;

pub use id::ID;

#[program]
pub mod event_stream {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, thread_id: Vec<u8>) -> Result<()> {
        initialize::handler(ctx, thread_id)
    }

    pub fn reset(ctx: Context<Reset>) -> Result<()> {
        initialize::reset(ctx)
    }

    pub fn ping(ctx: Context<Ping>) -> Result<()> {
        ping::handler(ctx)
    }

    pub fn process_event(ctx: Context<ProcessEvent>) -> Result<()> {
        process_event::handler(ctx)
    }
}
