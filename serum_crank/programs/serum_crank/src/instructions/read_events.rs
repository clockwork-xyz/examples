use {
    crate::state::*,
    anchor_lang::{
        system_program::{transfer, Transfer},
        prelude::*,
        solana_program::{system_program,instruction::Instruction},
    },
    anchor_spl::{dex::serum_dex::state::{strip_header, EventQueueHeader, Event, Queue as SerumDexQueue}, token::TokenAccount},
    clockwork_sdk::{queue_program::accounts::{Queue, QueueAccount}, CrankResponse}
};

#[derive(Accounts)]
pub struct ReadEvents<'info> {
    #[account(
        mut, 
        seeds = [SEED_CRANK, crank.market.as_ref()],
        bump, 
        has_one = event_queue,
        has_one = market,
        has_one = mint_a_vault,
        has_one = mint_b_vault,
    )]
    pub crank: Account<'info, Crank>,

    #[account(
        signer, 
        mut,
        address = crank_queue.pubkey(),
        constraint = crank_queue.id.eq("crank"),
    )]
    pub crank_queue: Account<'info, Queue>,

    #[account(address = anchor_spl::dex::ID)]
    pub dex_program: Program<'info, anchor_spl::dex::Dex>,

    /// CHECK: this account is validated against the crank account
    pub event_queue: AccountInfo<'info>,

    /// CHECK: this account is validated against the crank account
    pub market: AccountInfo<'info>,

    pub mint_a_vault: Account<'info, TokenAccount>,

    pub mint_b_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, ReadEvents<'info>>) -> Result<CrankResponse> {
    // Get accounts
    let crank = &mut ctx.accounts.crank;
    let crank_queue = &mut ctx.accounts.crank_queue;
    let dex_program = &ctx.accounts.dex_program;
    let event_queue = &ctx.accounts.event_queue;
    let market = &ctx.accounts.market;
    let mint_a_vault = &ctx.accounts.mint_a_vault;
    let mint_b_vault = &ctx.accounts.mint_b_vault;
    let payer = &mut ctx.accounts.payer;
    let system_program = &ctx.accounts.system_program;

    let mut next_ix_accounts = vec![
        AccountMeta::new_readonly(crank.key(), false),
        AccountMeta::new_readonly(crank_queue.key(), true),
        AccountMeta::new_readonly(dex_program.key(), false),
        AccountMeta::new(event_queue.key(), false),
        AccountMeta::new(market.key(), false),
        AccountMeta::new(mint_a_vault.key(), false),
        AccountMeta::new(mint_b_vault.key(), false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    // deserialize event queue
    let mut open_orders = Vec::new();

    let (header, buf) = 
        strip_header::<EventQueueHeader, Event>(event_queue, false).unwrap();
    let events = SerumDexQueue::new(header, buf);
    for event in events.iter() {
        // <https://github.com/rust-lang/rust/issues/82523>
        let val = unsafe { std::ptr::addr_of!(event.owner).read_unaligned() };
        let owner = Pubkey::new(safe_transmute::to_bytes::transmute_one_to_bytes(core::convert::identity(&val)));
        open_orders.push(owner);
        next_ix_accounts.push(AccountMeta::new(owner, false));
    }
        
    // write event queue data to crank account
    crank.open_orders = open_orders;

    // realloc memory for crank's account
    let new_size = 8 + crank.try_to_vec()?.len();
    crank.to_account_info().realloc(new_size, false)?;

    // pay rent if more space has been allocated
    let minimum_rent = Rent::get().unwrap().minimum_balance(new_size);

    if minimum_rent > crank.to_account_info().lamports() {
        transfer(
            CpiContext::new(
                system_program.to_account_info(),
                Transfer {
                    from: payer.to_account_info(),
                    to: crank.to_account_info(),
                },
            ),
            minimum_rent.checked_sub(crank.to_account_info().lamports()).unwrap()
        )?;
    }  
    
    // return consume events ix
    Ok(CrankResponse { 
        next_instruction: Some(
            Instruction {
                program_id: crate::ID,
                accounts: next_ix_accounts,
                data: clockwork_sdk::anchor_sighash("consume_events").into(),
            }
            .into()
        ),
        kickoff_instruction: None
    }) 
}
