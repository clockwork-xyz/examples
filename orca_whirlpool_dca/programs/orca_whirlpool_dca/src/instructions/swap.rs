use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::system_program,
    },
    anchor_spl::token::{Token, TokenAccount, transfer, Transfer},
    clockwork_sdk::state::{Thread, ThreadAccount},
};

#[derive(Accounts)]
pub struct Swap<'info> {
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
            dca.authority.as_ref(), 
            dca.a_mint.as_ref(), 
            dca.b_mint.as_ref(), 
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

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, Swap<'info>>) -> Result<()> {
    // get accounts
    let authority_a_vault = &mut ctx.accounts.authority_a_vault; 
    let authority_b_vault = &mut ctx.accounts.authority_b_vault; 
    let dca = &ctx.accounts.dca;
    let dca_a_vault= &mut ctx.accounts.dca_a_vault;
    let dca_b_vault= &mut ctx.accounts.dca_b_vault;
    let _dca_thread= &ctx.accounts.dca_thread;
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

    // get dca bump
    let bump = *ctx.bumps.get("dca").unwrap();

    // transfer swap amount from authority to dca ata
    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: if dca.a_to_b { authority_a_vault.to_account_info() } else { authority_b_vault.to_account_info() },
                to: if dca.a_to_b { dca_a_vault.to_account_info() } else { dca_b_vault.to_account_info() },
                authority: dca.to_account_info(),
            },
            &[&[
                SEED_DCA,
                dca.authority.as_ref(),
                dca.a_mint.as_ref(),
                dca.b_mint.as_ref(),
                &[bump],
            ]],
        ), 
        dca.amount
    )?;

    whirlpool::cpi::swap(
        CpiContext::new_with_signer(
            orca_whirlpool_program.to_account_info(), 
            whirlpool::cpi::accounts::Swap {
                token_program: token_program.to_account_info(),
                token_authority: dca.to_account_info(),
                whirlpool: whirlpool.to_account_info(),
                token_owner_account_a: dca_a_vault.to_account_info(),
                token_vault_a: whirlpool_token_a_vault.to_account_info(),
                token_owner_account_b: dca_b_vault.to_account_info(),
                token_vault_b: whirlpool_token_b_vault.to_account_info(),
                tick_array0: tick_array0.to_account_info(),
                tick_array1: tick_array1.to_account_info(),
                tick_array2: tick_array2.to_account_info(),
                oracle: oracle.to_account_info(),
            }, 
            &[&[
                SEED_DCA,
                dca.authority.as_ref(),
                dca.a_mint.as_ref(),
                dca.b_mint.as_ref(),
                &[bump],
            ]],
        ), 
        dca.amount, 
        dca.other_amount_threshold, 
        dca.sqrt_price_limit, 
        dca.amount_specified_is_input, 
        dca.a_to_b
    )?;

    
    // reload account after swap
    if dca.a_to_b { dca_b_vault.reload()? } else { dca_a_vault.reload()? }

     // settle funds back to user
    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: if dca.a_to_b { dca_b_vault.to_account_info() } else { dca_a_vault.to_account_info() },
                to: if dca.a_to_b { authority_b_vault.to_account_info() } else { authority_a_vault.to_account_info() },
                authority: dca.to_account_info(),
            },
            &[&[
                SEED_DCA,
                dca.authority.as_ref(),
                dca.a_mint.as_ref(),
                dca.b_mint.as_ref(),
                &[bump],
            ]]
        ),
        dca_b_vault.amount,
    )?;


    Ok(())
}