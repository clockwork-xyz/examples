use {
    anchor_lang::{
        prelude::*,
        solana_program::{instruction::Instruction, system_program},
        InstructionData,
    },
    anchor_spl::token::{Mint, Token, TokenAccount},
    clockwork_sdk::state::{Thread, ThreadAccount, ThreadResponse},
    whirlpool::{state::Whirlpool, utils::get_tick_array_pubkeys},
};

#[derive(Accounts)]
#[instruction(
    amount: u64,
    a_to_b: bool,
)]
pub struct OrcaWhirlpoolPreSwap<'info> {
    /// CHECK:
    pub a_mint: Box<Account<'info, Mint>>,

    /// CHECK:
    pub b_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = a_mint,
        associated_token::authority = swap_thread.authority
    )]
    pub authority_a_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = b_mint,
        associated_token::authority = swap_thread.authority
    )]
    pub authority_b_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::authority = swap_thread,
        associated_token::mint = a_mint,
    )]
    pub swap_thread_a_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::authority = swap_thread,
        associated_token::mint = b_mint,
    )]
    pub swap_thread_b_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        signer,
        address = swap_thread.pubkey(),
        constraint = swap_thread.authority == authority_a_vault.owner
    )]
    pub swap_thread: Box<Account<'info, Thread>>,

    /// CHECK:
    pub oracle: AccountInfo<'info>,

    /// CHECK:
    pub orca_whirlpool_program: AccountInfo<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, Token>,

    /// CHECK:
    #[account(mut)]
    pub whirlpool: Account<'info, Whirlpool>,

    /// CHECK:
    #[account(mut)]
    pub whirlpool_token_a_vault: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub whirlpool_token_b_vault: AccountInfo<'info>,
}

pub fn handler<'info>(
    ctx: Context<OrcaWhirlpoolPreSwap<'info>>,
    amount: u64,
    a_to_b: bool,
) -> Result<ThreadResponse> {
    // get accounts
    let whirlpool = &ctx.accounts.whirlpool;

    // get tick array pubkeys for next swap
    let tick_array_pubkeys = get_tick_array_pubkeys(
        whirlpool.tick_current_index,
        whirlpool.tick_spacing,
        a_to_b,
        &whirlpool::ID,
        &whirlpool.key(),
    );

    // return swap as the kickoff_instruction because this program is stateless
    Ok(ThreadResponse {
        kickoff_instruction: None,
        next_instruction: Some(
            Instruction {
                program_id: crate::ID,
                accounts: [
                    crate::accounts::OrcaWhirlpoolSwap {
                        a_mint: ctx.accounts.a_mint.key(),
                        b_mint: ctx.accounts.b_mint.key(),
                        authority_a_vault: ctx.accounts.authority_a_vault.key(),
                        authority_b_vault: ctx.accounts.authority_b_vault.key(),
                        swap_thread: ctx.accounts.swap_thread.key(),
                        swap_thread_a_vault: ctx.accounts.swap_thread_a_vault.key(),
                        swap_thread_b_vault: ctx.accounts.swap_thread_b_vault.key(),
                        oracle: ctx.accounts.oracle.key(),
                        system_program: ctx.accounts.system_program.key(),
                        token_program: ctx.accounts.token_program.key(),
                        whirlpool: ctx.accounts.whirlpool.key(),
                        orca_whirlpool_program: whirlpool::ID,
                        whirlpool_token_a_vault: whirlpool.token_vault_a,
                        whirlpool_token_b_vault: whirlpool.token_vault_b,
                    }
                    .to_account_metas(Some(true)),
                    // REMAINING ACCOUNTS
                    tick_array_pubkeys
                        .iter()
                        .map(|pk| AccountMeta::new(*pk, false))
                        .collect::<Vec<AccountMeta>>(),
                ]
                .concat(),
                data: crate::instruction::OrcaWhirlpoolSwap { amount, a_to_b }.data(),
            }
            .into(),
        ),
    })
}
