use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{system_program,instruction::Instruction},
    },
    anchor_lang::solana_program::program::invoke_signed,
    anchor_spl::{token::TokenAccount, dex::serum_dex},
    clockwork_crank::state::{CrankResponse, Queue, SEED_QUEUE},
};

#[derive(Accounts)]
pub struct ConsumeEvents<'info> {
    #[account(
        seeds = [SEED_CRANK, crank.market.key().as_ref()], 
        bump,
        has_one = market,
        has_one = event_queue,
        has_one = mint_a_vault,
        has_one = mint_b_vault,
    )]
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
    pub crank_queue: Account<'info, Queue>,
   
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

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, ConsumeEvents<'info>>) -> Result<CrankResponse> {
    // Get accounts
    let crank = &ctx.accounts.crank;
    let crank_queue = &ctx.accounts.crank_queue;
    let dex_program = &ctx.accounts.dex_program;
    let event_queue = &mut ctx.accounts.event_queue;
    let market = &mut ctx.accounts.market;
    let mint_a_vault = &mut ctx.accounts.mint_a_vault;
    let mint_b_vault = &mut ctx.accounts.mint_b_vault;

    // get crank bump
    let bump = *ctx.bumps.get("crank").unwrap();
    
    // coerce open orders type
    let open_orders_accounts = crank.open_orders.iter().map(|pk| pk).collect::<Vec<&Pubkey>>();
    
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
    }

    // return read events ix
    Ok(CrankResponse { 
        next_instruction: Some(
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
                    AccountMeta::new(clockwork_crank::payer::ID, true),
                    AccountMeta::new_readonly(system_program::ID, false),
                ],
                data: clockwork_crank::anchor::sighash("read_events").into(),
            }
            .into()
        )
    }) 
}
