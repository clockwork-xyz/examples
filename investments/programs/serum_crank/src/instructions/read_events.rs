
use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{system_program,instruction::Instruction},
    },
    anchor_spl::{dex::serum_dex::state::{strip_header, EventQueueHeader, Event, Queue}},
    clockwork_crank::state::{CrankResponse, Queue as ClockworkQueue, SEED_QUEUE},
};

#[derive(Accounts)]
pub struct ReadEvents<'info> {
    #[account(seeds = [SEED_CRANK], bump)]
    pub crank: Account<'info, Crank>,

    #[account(
        signer, 
        seeds = [
            SEED_QUEUE, 
            crank.key().as_ref(), 
            "crank".as_bytes()
        ], 
        seeds::program = clockwork_crank::ID,
        bump,
    )]
    pub crank_queue: Box<Account<'info, ClockworkQueue>>,

    #[account(address = anchor_spl::dex::ID)]
    pub dex_program: Program<'info, anchor_spl::dex::Dex>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, ReadEvents<'info>>) -> Result<CrankResponse> {
    // Get accounts
    let crank = &mut ctx.accounts.crank;
    let crank_queue = &ctx.accounts.crank_queue;
    let dex_program = &ctx.accounts.dex_program;

    // Get remaining accounts
    let market = ctx.remaining_accounts.get(0).unwrap();
    let mint_a_vault = ctx.remaining_accounts.get(1).unwrap();
    let mint_b_vault = ctx.remaining_accounts.get(2).unwrap();
    let event_queue = ctx.remaining_accounts.get(3).unwrap();

    // deserialize event queue
    let mut open_orders = Vec::new(); 

    msg!("events: [");
    let (header, buf) = 
        strip_header::<EventQueueHeader, Event>(event_queue, false).unwrap();
    let events = Queue::new(header, buf);
    for event in events.iter() {
        // context for brw & val <https://github.com/rust-lang/rust/issues/82523>
        let brw = std::ptr::addr_of!(event.owner);
        let val = unsafe { brw.read_unaligned() };
        let owner = Pubkey::new(safe_transmute::to_bytes::transmute_one_to_bytes(core::convert::identity(&val)));
        msg!("owner: {}", owner.to_string());
        msg!("     {:?},", event.as_view().unwrap());
        open_orders.push(owner);
    }
    msg!("]");
    
    // write event queue data to crank account
    crank.index(open_orders)?; 
  
    // return event queue ix to read and write to crank account for next consume events invocation
    Ok(CrankResponse { 
        next_instruction: Some(
            Instruction {
                program_id: crate::ID,
                accounts: vec![
                AccountMeta::new_readonly(crank.key(), false),
                AccountMeta::new_readonly(crank_queue.key(), true),
                AccountMeta::new_readonly(dex_program.key(), false),
                AccountMeta::new_readonly(system_program::ID, false),
                // Extra Accounts
                AccountMeta::new(market.key(), false),
                AccountMeta::new(mint_a_vault.key(), false),
                AccountMeta::new(mint_b_vault.key(), false),
                AccountMeta::new(event_queue.key(), false),
                ],
                data: clockwork_crank::anchor::sighash("consume_events").into(),
            }
            .into()
        )
    }) 
}
