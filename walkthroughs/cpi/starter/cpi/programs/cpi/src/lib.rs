mod id;
use id::ID;
use anchor_lang::{
    prelude::*,
    InstructionData,
    solana_program::{instruction::Instruction, system_program},
};

#[program]
pub mod cpi {
    use super::*;

    pub fn hello_ix(_ctx: Context<Hello>) -> Result<()> {
        msg!("Hello! The current time is: {}", Clock::get().unwrap().unix_timestamp);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Hello{}
