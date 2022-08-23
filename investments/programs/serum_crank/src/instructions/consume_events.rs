
use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{program::invoke_signed, system_program, instruction::Instruction}
        
    },
    anchor_spl::dex::serum_dex,
    clockwork_crank::{
        state::{SEED_QUEUE, Queue, CrankResponse},
    },
};

#[derive(Accounts)]
pub struct ConsumeEvents<'info> {
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
    pub crank_queue: Account<'info, Queue>,
   
    #[account(address = anchor_spl::dex::ID)]
    pub dex_program: Program<'info, anchor_spl::dex::Dex>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, ConsumeEvents<'info>>) -> Result<CrankResponse> {
    // Get accounts
    let crank = &ctx.accounts.crank;
    let crank_queue = &ctx.accounts.crank_queue;
    let dex_program = &ctx.accounts.dex_program;

    // Get remaining accounts
    let market = ctx.remaining_accounts.get(0).unwrap();
    let mint_a_vault = ctx.remaining_accounts.get(1).unwrap();
    let mint_b_vault = ctx.remaining_accounts.get(2).unwrap();
    let event_queue = ctx.remaining_accounts.get(3).unwrap();

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
                std::u16::MAX).unwrap();

        let account_infos = &[
            dex_program.to_account_info(), 
            market.to_account_info(), 
            event_queue.to_account_info(),
            mint_b_vault.to_account_info(), 
            mint_a_vault.to_account_info()
        ];

        // invoke crank events ix
        invoke_signed(&consume_events_ix,account_infos,&[&[SEED_CRANK, &[bump]]])?;
    }

    // return read events ix
    Ok(CrankResponse { 
        next_instruction: Some(
            Instruction {
                program_id: crate::ID,
                accounts: vec![
                    AccountMeta::new_readonly(crank.key(), false),
                    AccountMeta::new(crank_queue.key(), true),
                    AccountMeta::new_readonly(dex_program.key(), false),
                    AccountMeta::new_readonly(system_program::ID, false),
                    // Extra Accounts
                    AccountMeta::new(market.key(), false),
                    AccountMeta::new(mint_a_vault.key(), false),
                    AccountMeta::new(mint_b_vault.key(), false),
                    AccountMeta::new(event_queue.key(), false),
                ],
                data: clockwork_crank::anchor::sighash("read_events").into(),
            }
            .into()
        )
    }) 
}
