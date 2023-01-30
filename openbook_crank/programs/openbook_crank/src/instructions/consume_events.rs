use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{   
            instruction::Instruction, 
            system_program, 
            program::invoke_signed
        },
    },
    anchor_spl::{
        dex::serum_dex::{
            instruction::consume_events,
            state::{strip_header, Event, EventQueueHeader, Queue},
        },
        token::TokenAccount,
    },
    clockwork_sdk::state::{InstructionData, Thread, ThreadAccount, ThreadResponse},
};

#[derive(Accounts)]
pub struct ConsumeEvents<'info> {
    #[account(
        seeds = [
            SEED_CRANK, 
            crank.authority.as_ref(), 
            crank.market.as_ref(), 
        ],
        bump,
        has_one = market,
    )]
    pub crank: Box<Account<'info, Crank>>,

    #[account(
        signer,
        address = crank_thread.pubkey(),
        constraint = crank_thread.authority == crank.authority
    )]
    pub crank_thread: Box<Account<'info, Thread>>,

    pub dex_program: Program<'info, OpenBookDex>,

    /// CHECK: this account is validated against the crank account
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,

    /// CHECK: this account is validated against the crank account
    #[account(mut)]
    pub market: AccountInfo<'info>,

    #[account(mut)]
    pub coin_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub pc_vault: Box<Account<'info, TokenAccount>>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, ConsumeEvents<'info>>,
) -> Result<ThreadResponse> {
    // get accounts
    let crank = &ctx.accounts.crank;
    let crank_thread = &ctx.accounts.crank_thread;
    let dex_program = &ctx.accounts.dex_program;
    let event_queue = &mut ctx.accounts.event_queue;
    let market = &mut ctx.accounts.market;
    let coin_vault = &mut ctx.accounts.coin_vault;
    let pc_vault = &mut ctx.accounts.pc_vault;
    let open_orders_account_infos = ctx.remaining_accounts.clone().to_vec();

    // get crank bump
    let bump = *ctx.bumps.get("crank").unwrap();

    const CONSUME_EVENTS_ACC_LEN: usize = 8;
    const MAX_OPEN_ORDER_ACC_LEN: usize = 5;

    let mut consume_events_account_metas = vec![
        AccountMeta::new_readonly(crank.key(), false),
        AccountMeta::new_readonly(crank_thread.key(), true),
        AccountMeta::new_readonly(dex_program.key(), false),
        AccountMeta::new(event_queue.key(), false),
        AccountMeta::new(market.key(), false),
        AccountMeta::new(coin_vault.key(), false),
        AccountMeta::new(pc_vault.key(), false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    // deserialize event queue
    let (header, buf) = 
        strip_header::<EventQueueHeader, Event>(&event_queue, false).unwrap();
    let events = Queue::new(header, buf);
    for event in events.iter() {
        // <https://github.com/rust-lang/rust/issues/82523>
        let val = unsafe { std::ptr::addr_of!(event.owner).read_unaligned() };
        let owner = Pubkey::new(safe_transmute::to_bytes::transmute_one_to_bytes(
            core::convert::identity(&val),
        ));
        consume_events_account_metas.push(AccountMeta::new(owner, false));

        if consume_events_account_metas.len() >= CONSUME_EVENTS_ACC_LEN + MAX_OPEN_ORDER_ACC_LEN {
            break;
        }
    }

    drop(events);

    let consume_events_ix: Option<InstructionData> = Some(
        Instruction {
            program_id: crate::ID,
            accounts: consume_events_account_metas.clone(),
            data: clockwork_sdk::utils::anchor_sighash("consume_events").into(),
        }
        .into(),
    );

    if open_orders_account_infos.len() > 0 {
        msg!("open orders account to be consumed: {}", open_orders_account_infos.len());

        // derive consume events ix
        let consume_events_openbook_ix = consume_events(
            &dex_program.key(),
            open_orders_account_infos
                .clone()
                .iter()
                .map(|acc| acc.key())
                .collect::<Vec<Pubkey>>()
                .iter()
                .collect::<Vec<&Pubkey>>(),
            &market.key(),
            &event_queue.key(),
            &coin_vault.key(),
            &pc_vault.key(),
            crank.limit,
        ).unwrap();

        let mut consume_events_account_infos = vec![
            market.to_account_info(),
            event_queue.to_account_info(),
            coin_vault.to_account_info(),
            pc_vault.to_account_info(),
            dex_program.to_account_info(),
        ];

        consume_events_account_infos.append(&mut open_orders_account_infos.clone());

        // invoke consume events ix
        invoke_signed(
            &consume_events_openbook_ix,
            &consume_events_account_infos,
            &[&[
                SEED_CRANK,
                crank.authority.as_ref(),
                crank.market.as_ref(),
                &[bump],
            ]],
        )?;
        
    }

    if consume_events_account_metas.len() > CONSUME_EVENTS_ACC_LEN {
        return Ok(ThreadResponse {
            kickoff_instruction: None,
            next_instruction: consume_events_ix,
        });
    } else {
        return Ok(ThreadResponse {
            kickoff_instruction: consume_events_ix,
            next_instruction: None,
        });
    }
}
