use anchor_lang::solana_program::instruction::Instruction;

use {
    anchor_lang::{prelude::*, solana_program::system_program, InstructionData},
    anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer},
    clockwork_sdk::state::{Thread, ThreadAccount, ThreadResponse},
    whirlpool::utils::get_tick_array_pubkeys,
};

#[derive(Accounts)]
#[instruction(amount: u64, a_to_b: bool)]
pub struct OrcaWhirlpoolSwap<'info> {
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
    pub whirlpool: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub whirlpool_token_a_vault: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub whirlpool_token_b_vault: AccountInfo<'info>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, OrcaWhirlpoolSwap<'info>>,
    amount: u64,
    a_to_b: bool,
) -> Result<ThreadResponse> {
    // get accounts
    let authority_a_vault = &mut ctx.accounts.authority_a_vault;
    let authority_b_vault = &mut ctx.accounts.authority_b_vault;
    let swap_thread_a_vault = &mut ctx.accounts.swap_thread_a_vault;
    let swap_thread_b_vault = &mut ctx.accounts.swap_thread_b_vault;
    let swap_thread = &ctx.accounts.swap_thread;
    let oracle = &ctx.accounts.oracle;
    let orca_whirlpool_program = &ctx.accounts.orca_whirlpool_program;
    let token_program = &ctx.accounts.token_program;
    let whirlpool = &mut ctx.accounts.whirlpool;
    let whirlpool_token_a_vault = &mut ctx.accounts.whirlpool_token_a_vault;
    let whirlpool_token_b_vault = &mut ctx.accounts.whirlpool_token_b_vault;

    // get remaining accounts
    let tick_array0 = ctx.remaining_accounts.get(0).unwrap();
    let tick_array1 = ctx.remaining_accounts.get(1).unwrap();
    let tick_array2 = ctx.remaining_accounts.get(2).unwrap();

    let whirlpool_data = whirlpool.try_borrow_data().unwrap().to_owned();
    let whirlpool_state =
        whirlpool::state::Whirlpool::try_deserialize(&mut whirlpool_data.as_slice()).unwrap();

    // get tick array pubkeys
    let tick_array_pubkeys = get_tick_array_pubkeys(
        whirlpool_state.tick_current_index,
        whirlpool_state.tick_spacing,
        a_to_b,
        &whirlpool::ID,
        &whirlpool.key(),
    );

    // transfer swap amount from authority to swap_thread ata
    transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: if a_to_b {
                    authority_a_vault.to_account_info()
                } else {
                    authority_b_vault.to_account_info()
                },
                to: if a_to_b {
                    swap_thread_a_vault.to_account_info()
                } else {
                    swap_thread_b_vault.to_account_info()
                },
                authority: swap_thread.to_account_info(),
            },
        ),
        amount,
    )?;

    whirlpool::cpi::swap(
        CpiContext::new(
            orca_whirlpool_program.to_account_info(),
            whirlpool::cpi::accounts::Swap {
                token_program: token_program.to_account_info(),
                token_authority: swap_thread.to_account_info(),
                whirlpool: whirlpool.to_account_info(),
                token_owner_account_a: swap_thread_a_vault.to_account_info(),
                token_vault_a: whirlpool_token_a_vault.to_account_info(),
                token_owner_account_b: swap_thread_b_vault.to_account_info(),
                token_vault_b: whirlpool_token_b_vault.to_account_info(),
                tick_array0: tick_array0.to_account_info(),
                tick_array1: tick_array1.to_account_info(),
                tick_array2: tick_array2.to_account_info(),
                oracle: oracle.to_account_info(),
            },
        ),
        amount,
        0,
        whirlpool_state.sqrt_price,
        false,
        a_to_b,
    )?;

    // reload account after swap
    if a_to_b {
        swap_thread_b_vault.reload()?
    } else {
        swap_thread_a_vault.reload()?
    }

    // settle funds back to user
    transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: if a_to_b {
                    swap_thread_b_vault.to_account_info()
                } else {
                    swap_thread_a_vault.to_account_info()
                },
                to: if a_to_b {
                    authority_b_vault.to_account_info()
                } else {
                    authority_a_vault.to_account_info()
                },
                authority: swap_thread.to_account_info(),
            },
        ),
        swap_thread_b_vault.amount,
    )?;

    Ok(ThreadResponse {
        kickoff_instruction: Some(
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
                        whirlpool_token_a_vault: whirlpool_state.token_vault_a,
                        whirlpool_token_b_vault: whirlpool_state.token_vault_b,
                    }
                    .to_account_metas(Some(true)),
                    // REMAINING ACCOUNTS
                    tick_array_pubkeys
                        .iter()
                        .map(|pk| AccountMeta::new_readonly(*pk, false))
                        .collect::<Vec<AccountMeta>>(),
                ]
                .concat(),
                data: crate::instruction::OrcaWhirlpoolSwap {
                    amount: 10_000_000_000,
                    a_to_b: true,
                }
                .data(),
            }
            .into(),
        ),
        next_instruction: None,
    })
}

// #[inline(always)]
// fn to_account_metas_datas(
//     context: &Context<OrcaWhirlpoolSwap>,
// ) -> Vec<clockwork_sdk::state::AccountMetaData> {
//     let mut account_metas = Vec::with_capacity(14);
//     account_metas.push(AccountMetaData::new_readonly(
//         context.accounts.a_mint.key(),
//         false,
//     ));
//     account_metas.push(AccountMetaData::new_readonly(
//         context.accounts.b_mint.key(),
//         false,
//     ));
//     account_metas.push(AccountMetaData::new(
//         context.accounts.authority_a_vault.key(),
//         false,
//     ));
//     account_metas.push(AccountMetaData::new(
//         context.accounts.authority_b_vault.key(),
//         false,
//     ));
//     account_metas.push(AccountMetaData::new(
//         context.accounts.swap_thread_a_vault.key(),
//         false,
//     ));
//     account_metas.push(AccountMetaData::new(
//         context.accounts.swap_thread_b_vault.key(),
//         false,
//     ));
//     account_metas.push(AccountMetaData::new_readonly(
//         context.accounts.swap_thread.key(),
//         true,
//     ));
//     account_metas.push(AccountMetaData::new_readonly(
//         context.accounts.oracle.key(),
//         false,
//     ));
//     account_metas.push(AccountMetaData::new_readonly(
//         context.accounts.orca_whirlpool_program.key(),
//         false,
//     ));
//     account_metas.push(AccountMetaData::new_readonly(
//         context.accounts.system_program.key(),
//         false,
//     ));
//     account_metas.push(AccountMetaData::new_readonly(
//         context.accounts.token_program.key(),
//         false,
//     ));
//     account_metas.push(AccountMetaData::new(whirlpool::ID, false));
//     account_metas.push(AccountMetaData::new(
//         context.accounts.whirlpool_token_a_vault.key(),
//         false,
//     ));
//     account_metas.push(AccountMetaData::new(
//         context.accounts.whirlpool_token_b_vault.key(),
//         false,
//     ));
//     account_metas
// }

// The sighash of a named instruction in an Anchor program.
// fn anchor_sighash(name: &str, mut data: Vec<u8>) -> Vec<u8> {
//     let preimage = format!("{}:{}", "global", name);
//     let mut sighash = [0u8; 8];
//     sighash.copy_from_slice(
//         &anchor_lang::solana_program::hash::hash(preimage.as_bytes()).to_bytes()[..8],
//     );
//     let mut d = sighash.to_vec();
//     d.append(&mut data);
//     d
// }

// crate::accounts::OrcaWhirlpoolSwap {
//     a_mint: ctx.accounts.a_mint.key(),
//     b_mint: ctx.accounts.b_mint.key(),
//     authority_a_vault: ctx.accounts.authority_a_vault.key(),
//     authority_b_vault: ctx.accounts.authority_b_vault.key(),
//     swap_thread: ctx.accounts.swap_thread.key(),
//     swap_thread_a_vault: ctx.accounts.swap_thread_a_vault.key(),
//     swap_thread_b_vault: ctx.accounts.swap_thread_b_vault.key(),
//     oracle: ctx.accounts.oracle.key(),
//     system_program: ctx.accounts.system_program.key(),
//     token_program: ctx.accounts.token_program.key(),
//     whirlpool: ctx.accounts.whirlpool.key(),
//     orca_whirlpool_program: whirlpool::ID,
//     whirlpool_token_a_vault: whirlpool_state.token_vault_a,
//     whirlpool_token_b_vault: whirlpool_state.token_vault_b,
// }
// .to_account_metas(Some(true)),
// REMAINING ACCOUNTS
