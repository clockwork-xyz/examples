use {
    crate::state::*,
    anchor_lang::solana_program::program::invoke_signed,
    anchor_lang::{
        prelude::*,
        solana_program::{instruction::Instruction, system_program},
    },
    anchor_spl::{dex::serum_dex, token::TokenAccount},
    clockwork_sdk::{
        state::{InstructionData, Thread, ThreadAccount, ThreadResponse},
        utils::PAYER_PUBKEY,
    },
};

#[derive(Accounts)]
pub struct ConsumeEvents<'info> {
    #[account(
        seeds = [SEED_CRANK, crank.authority.as_ref(), crank.market.as_ref(), crank.id.as_bytes()],
        bump,
        has_one = market,
        has_one = event_queue,
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
    pub mint_a_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub mint_b_vault: Account<'info, TokenAccount>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, ConsumeEvents<'info>>,
) -> Result<ThreadResponse> {
    // Get accounts
    let crank = &ctx.accounts.crank;
    let crank_thread = &ctx.accounts.crank_thread;
    let dex_program = &ctx.accounts.dex_program;
    let event_queue = &mut ctx.accounts.event_queue;
    let market = &mut ctx.accounts.market;
    let mint_a_vault = &mut ctx.accounts.mint_a_vault;
    let mint_b_vault = &mut ctx.accounts.mint_b_vault;
    let open_orders_account_infos = ctx.remaining_accounts.clone().to_vec();

    // get crank bump
    let bump = *ctx.bumps.get("crank").unwrap();

    // derive read events ix
    let read_events_ix: Option<InstructionData> = Some(
        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(crank.key(), false),
                AccountMeta::new(crank_thread.key(), true),
                AccountMeta::new_readonly(dex_program.key(), false),
                AccountMeta::new_readonly(event_queue.key(), false),
                AccountMeta::new_readonly(market.key(), false),
                AccountMeta::new(PAYER_PUBKEY, true),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: clockwork_sdk::utils::anchor_sighash("read_events").into(),
        }
        .into(),
    );

    let open_orders_account_pubkeys = &open_orders_account_infos
        .iter()
        .map(|acc| acc.key())
        .collect::<Vec<Pubkey>>();

    msg!("open order accs len: {}", open_orders_account_pubkeys.len());

    // if there are orders that need to be cranked
    if open_orders_account_pubkeys.len() > 0 {
        // derive consume events ix
        let consume_events_ix = serum_dex::instruction::consume_events(
            &dex_program.key(),
            open_orders_account_pubkeys.iter().collect::<Vec<&Pubkey>>(),
            &market.key(),
            &event_queue.key(),
            &mint_b_vault.key(),
            &mint_a_vault.key(),
            crank.limit,
        )
        .unwrap();

        let mut consume_events_account_infos = vec![
            market.to_account_info(),
            event_queue.to_account_info(),
            mint_a_vault.to_account_info(),
            mint_b_vault.to_account_info(),
            dex_program.to_account_info(),
        ];

        consume_events_account_infos.append(&mut open_orders_account_infos.clone());

        // invoke crank events ix
        invoke_signed(
            &consume_events_ix,
            &consume_events_account_infos,
            &[&[
                SEED_CRANK,
                crank.authority.as_ref(),
                crank.market.as_ref(),
                crank.id.as_bytes(),
                &[bump],
            ]],
        )?;

        // read events again bc there might be more open orders
        return Ok(ThreadResponse {
            kickoff_instruction: None,
            next_instruction: read_events_ix,
        });
    }

    // end execution context because there are no more events to consume
    Ok(ThreadResponse {
        kickoff_instruction: read_events_ix,
        next_instruction: None,
    })
}
