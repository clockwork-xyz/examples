use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{system_program,instruction::Instruction},
    },
    anchor_lang::solana_program::program::invoke_signed,
    anchor_spl::{token::TokenAccount, dex::serum_dex},
    clockwork_sdk::{thread_program::accounts::{Thread, ThreadAccount}, ExecResponse},
};

#[derive(Accounts)]
pub struct ConsumeEvents<'info> {
    #[account(
        address = Crank::pubkey(crank.market.key()),
        has_one = market,
        has_one = event_queue,
        has_one = mint_a_vault,
        has_one = mint_b_vault,
    )]
    pub crank: Box<Account<'info, Crank>>,

    #[account(
        signer, 
        address = crank_thread.pubkey(),
        constraint = crank_thread.id.eq("crank"),
    )]
    pub crank_thread: Box<Account<'info, Thread>>,
   
    #[account(address = anchor_spl::dex::ID)]
    pub dex_program: Program<'info, anchor_spl::dex::Dex>,

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

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, ConsumeEvents<'info>>) -> Result<ExecResponse> {
    // Get accounts
    let crank = &ctx.accounts.crank;
    let crank_thread = &ctx.accounts.crank_thread;
    let dex_program = &ctx.accounts.dex_program;
    let event_queue = &mut ctx.accounts.event_queue;
    let market = &mut ctx.accounts.market;
    let mint_a_vault = &mut ctx.accounts.mint_a_vault;
    let mint_b_vault = &mut ctx.accounts.mint_b_vault;

    // get crank bump
    let bump = *ctx.bumps.get("crank").unwrap();

    // derive read events ix
    let read_events_ix: Option<InstructionData> = 
        Some(
            Instruction {
                program_id: crate::ID,
                accounts: vec![
                    AccountMeta::new(crank.key(), false),
                    AccountMeta::new(crank_queue.key(), true),
                    AccountMeta::new_readonly(dex_program.key(), false),
                    AccountMeta::new_readonly(event_queue.key(), false),
                    AccountMeta::new_readonly(market.key(), false),
                    AccountMeta::new_readonly(mint_a_vault.key(), false),
                    AccountMeta::new_readonly(mint_b_vault.key(), false),
                    AccountMeta::new(PAYER_PUBKEY, true),
                    AccountMeta::new_readonly(system_program::ID, false),
                ],
                data: clockwork_sdk::anchor_sighash("read_events").into(),
            }
            .into()
        );
    
    // coerce open orders type
    let open_orders_accounts = crank.open_orders.iter().map(|pk| pk).collect::<Vec<&Pubkey>>();
    
    // if there are orders that need to be cranked
    if open_orders_accounts.len() > 0 {   
        // derive consume events ix
        let consume_events_ix = 
            serum_dex::instruction::consume_events(
                &dex_program.key(),
                open_orders_accounts,
                &market.key(), 
                &event_queue.key(),
                &mint_b_vault.key(), 
                &mint_a_vault.key(), 
                crank.limit
            ).unwrap();

        // construct account infos vec
        let mut cpi_account_infos = ctx.remaining_accounts.clone().to_vec(); 
        let mut rest_account_infos = vec![
            market.to_account_info(), 
            event_queue.to_account_info(), 
            mint_a_vault.to_account_info(), 
            mint_b_vault.to_account_info(), 
            dex_program.to_account_info()
        ];
        cpi_account_infos.append(&mut rest_account_infos);

        // invoke crank events ix
        invoke_signed(&consume_events_ix, &cpi_account_infos,&[&[SEED_CRANK, crank.market.as_ref(), &[bump]]])?;

        // read events again bc there might be more open orders
        return Ok(ExecResponse { 
            kickoff_instruction: None,
            next_instruction: read_events_ix
        });     
    }
    
    // end execution context because there are no more events to consume
    Ok(ExecResponse { 
        kickoff_instruction: read_events_ix,
        next_instruction: None,
    })

}
