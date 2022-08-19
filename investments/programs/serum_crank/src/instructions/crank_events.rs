use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{
            instruction::Instruction, native_token::LAMPORTS_PER_SOL, system_program,
        },
    },
    anchor_spl::dex::serum_dex::state::EventQueue,
    clockwork_crank::{
        program::ClockworkCrank,
        state::{Trigger, SEED_QUEUE},
    },
    std::mem::size_of,
};

#[derive(Accounts)]
pub struct CrankEvents<'info> {}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, CrankEvents<'info>>) -> Result<()> {
    // Get accounts
    // Get extra accounts
    let event_q = *ctx.remaining_accounts.get(0).unwrap();

    // deserialize event queue
    let event_q_data = event_q.deserialize_data::<EventQueue>().unwrap();

    // check if the event queue has events that need to be consumed
    if !event_q_data.empty() {
        // consume events and settle funds ser
    }

    Ok(())
}
