use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{instruction::Instruction, system_program},
    },
    anchor_spl::dex::serum_dex::state::{strip_header, Event, EventQueueHeader, Queue},
    clockwork_sdk::state::{Thread, ThreadResponse, ThreadAccount},
};

#[derive(Accounts)]
pub struct ReadEvents<'info> {
    #[account(
        mut, 
        seeds = [SEED_CRANK, crank.authority.as_ref(), crank.market.as_ref(), crank.id.as_bytes()],
        bump, 
        has_one = event_queue,
        has_one = market,
    )]
    pub crank: Box<Account<'info, Crank>>,

    #[account(
        signer,
        constraint = crank_thread.authority == crank.authority,
        address = crank_thread.pubkey()
    )]
    pub crank_thread: Box<Account<'info, Thread>>,

    pub dex_program: Program<'info, OpenBookDex>,

    /// CHECK: this account is validated against the crank account
    pub event_queue: AccountInfo<'info>,

    /// CHECK: this account is validated against the crank account
    pub market: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, ReadEvents<'info>>,
) -> Result<ThreadResponse> {
    // Get accounts
    let crank = &mut ctx.accounts.crank;
    let crank_thread = &mut ctx.accounts.crank_thread;
    let dex_program = &ctx.accounts.dex_program;
    let event_queue = &ctx.accounts.event_queue;
    let market = &ctx.accounts.market;
    
    let mut next_ix_accounts = vec![
        AccountMeta::new_readonly(crank.key(), false),
        AccountMeta::new_readonly(crank_thread.key(), true),
        AccountMeta::new_readonly(dex_program.key(), false),
        AccountMeta::new(event_queue.key(), false),
        AccountMeta::new(market.key(), false),
        AccountMeta::new(crank.mint_a_vault, false),
        AccountMeta::new(crank.mint_b_vault, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    // deserialize event queue
    let (header, buf) = strip_header::<EventQueueHeader, Event>(event_queue, false).unwrap();
    let events = Queue::new(header, buf);
    for event in events.iter() {
        // <https://github.com/rust-lang/rust/issues/82523>
        let val = unsafe { std::ptr::addr_of!(event.owner).read_unaligned() };
        let owner = Pubkey::new(safe_transmute::to_bytes::transmute_one_to_bytes(
            core::convert::identity(&val),
        ));
        // open_orders.push(owner);
        next_ix_accounts.push(AccountMeta::new(owner, false));
    }

    // return consume events ix
    Ok(ThreadResponse {
        kickoff_instruction: None,
        next_instruction: Some(
            Instruction {
                program_id: crate::ID,
                accounts: next_ix_accounts,
                data: clockwork_sdk::utils::anchor_sighash("consume_events").into(),
            }
            .into(),
        ),
    })
}
