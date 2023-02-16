use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{system_program, instruction::Instruction},
    },
    anchor_spl::token::{Token, TokenAccount},
    clockwork_sdk::state::{Thread, ThreadAccount, ThreadResponse},
    whirlpool::{state::Whirlpool, utils::get_tick_array_pubkeys},
};

#[derive(Accounts)]
pub struct GetTickArrays<'info> {
    #[account(
        mut,
        associated_token::mint = dca.a_mint,
        associated_token::authority = dca.authority
    )]
    pub authority_a_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = dca.b_mint,
        associated_token::authority = dca.authority
    )]
    pub authority_b_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [
            SEED_DCA,
            dca.authority.key().as_ref(), 
            dca.a_mint.key().as_ref(), 
            dca.b_mint.key().as_ref(), 
        ], 
        bump,
    )]
    pub dca: Box<Account<'info, Dca>>,

    #[account(
        mut,
        associated_token::authority = dca,
        associated_token::mint = dca.a_mint,
    )]
    pub dca_a_vault: Box<Account<'info, TokenAccount>>,
    
    #[account(
        mut,
        associated_token::authority = dca,
        associated_token::mint = dca.b_mint,
    )]
    pub dca_b_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        signer,
        address = dca_thread.pubkey(),
        constraint = dca_thread.authority == dca.authority
    )]
    pub dca_thread: Box<Account<'info, Thread>>,

    /// CHECK:
    pub oracle: AccountInfo<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, Token>,

    /// CHECK: 
    pub whirlpool: AccountInfo<'info>,
}

pub fn handler<'a, 'info>(ctx: Context<GetTickArrays<'info>>) -> Result<ThreadResponse> {
    let dca = &ctx.accounts.dca;

    // deserialize whirlpool state
    let whirlpool_data = ctx.accounts.whirlpool.try_borrow_data().unwrap().to_owned();
    let whirlpool_state = Whirlpool::try_deserialize(&mut whirlpool_data.as_slice()).unwrap();

    msg!(" whirlpool current idx: {}", whirlpool_state.tick_current_index);
    msg!(" whirlpool tick spacing: {}", whirlpool_state.tick_spacing);
    msg!("  whirlpool sqrt price: {}", whirlpool_state.sqrt_price);

    // get tick array pubkeys
    let tick_array_pubkeys = 
        get_tick_array_pubkeys(
            whirlpool_state.tick_current_index, 
            whirlpool_state.tick_spacing, 
            dca.a_to_b,
            &whirlpool::ID,
            &dca.whirlpool
        );

    // thread response with swap next_instruction
    Ok(
      ThreadResponse { 
        kickoff_instruction: None, 
        next_instruction: Some(Instruction {
            program_id: crate::ID,
            accounts: [
                crate::accounts::Swap { 
                    authority_a_vault: ctx.accounts.authority_a_vault.key(), 
                    authority_b_vault: ctx.accounts.authority_b_vault.key(), 
                    dca: ctx.accounts.dca.key(), 
                    dca_a_vault: ctx.accounts.dca_a_vault.key(), 
                    dca_b_vault: ctx.accounts.dca_b_vault.key(), 
                    dca_thread: ctx.accounts.dca_thread.key(), 
                    oracle: ctx.accounts.oracle.key(), 
                    orca_whirlpool_program: whirlpool::ID, 
                    system_program: ctx.accounts.system_program.key(), 
                    token_program: ctx.accounts.token_program.key(), 
                    whirlpool: ctx.accounts.whirlpool.key(),
                    whirlpool_token_a_vault: whirlpool_state.token_vault_a,
                    whirlpool_token_b_vault: whirlpool_state.token_vault_b, 
                }.to_account_metas(Some(true)),
                tick_array_pubkeys
                    .iter()
                    .map(|pk| AccountMeta::new(*pk, false))
                    .collect::<Vec<AccountMeta>>()
            ].concat(),
            data: clockwork_sdk::utils::anchor_sighash("swap").to_vec(),
        }.into())
      }
    )
}   