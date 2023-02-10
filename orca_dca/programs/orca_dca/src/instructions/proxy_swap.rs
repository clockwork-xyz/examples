use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{system_program, program::invoke_signed, instruction::Instruction},
    },
    anchor_spl::token::{Token, TokenAccount, transfer, Transfer},
    clockwork_sdk::state::{Thread, ThreadAccount},
    spl_token_swap::instruction::{SwapInstruction, Swap}
};

#[derive(Accounts)]
pub struct ProxySwap<'info> {
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
    pub orca_swap_program: AccountInfo<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, Token>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, ProxySwap<'info>>) -> Result<()> {
    // get accounts
    let authority_a_vault = &mut ctx.accounts.authority_a_vault; 
    let authority_b_vault = &mut ctx.accounts.authority_b_vault; 
    let dca = &ctx.accounts.dca;
    let dca_a_vault= &mut ctx.accounts.dca_a_vault;
    let dca_b_vault= &mut ctx.accounts.dca_b_vault;
    let orca_swap_program = &ctx.accounts.orca_swap_program;
    let token_program = &ctx.accounts.token_program;

    // get remaining accounts
    let address = ctx.remaining_accounts.get(0).unwrap();
    let authority = ctx.remaining_accounts.get(1).unwrap();
    let pool_source = ctx.remaining_accounts.get(2).unwrap();
    let pool_destination = ctx.remaining_accounts.get(3).unwrap();
    let pool_token_mint = ctx.remaining_accounts.get(4).unwrap();
    let fee_account = ctx.remaining_accounts.get(5).unwrap();
    
    // get dca bump
    let bump = *ctx.bumps.get("dca").unwrap();

    // transfer swap amount from authority to dca ata
    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: authority_a_vault.to_account_info(),
                to: dca_a_vault.to_account_info(),
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
        dca.amount_in,
    )?;

    // perform swap on orca pool
    invoke_signed(
        &Instruction {
            program_id: orca_swap_program.key(), 
            accounts: vec![
                AccountMeta::new_readonly(address.key(), false),
                AccountMeta::new_readonly(authority.key(), false),
                AccountMeta::new_readonly(dca.key(), true),
                AccountMeta::new(dca_a_vault.key(), false),
                AccountMeta::new(pool_source.key(), false),
                AccountMeta::new(pool_destination.key(), false),
                AccountMeta::new(dca_b_vault.key(), false),
                AccountMeta::new(pool_token_mint.key(), false),
                AccountMeta::new(fee_account.key(), false),
                AccountMeta::new_readonly(token_program.key(), false),
            ], 
            data: SwapInstruction::Swap(
                Swap { 
                    amount_in: dca.amount_in, 
                    minimum_amount_out: dca.minimum_amount_out
                }).pack() 
            },
        &[
            orca_swap_program.to_account_info(),
            address.to_account_info(),
            authority.to_account_info(),
            dca.to_account_info(),
            dca_a_vault.to_account_info(),
            pool_source.to_account_info(),
            pool_destination.to_account_info(),
            dca_b_vault.to_account_info(),
            pool_token_mint.to_account_info(),
            fee_account.to_account_info(),
            token_program.to_account_info(),
        ], 
        &[&[
                SEED_DCA,
                dca.authority.as_ref(),
                dca.a_mint.as_ref(),
                dca.b_mint.as_ref(),
                &[bump],
            ]]
    )?;

    
    // reload account after swap
    dca_b_vault.reload()?;

     // settle funds back to user
    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: dca_b_vault.to_account_info(),
                to: authority_b_vault.to_account_info(),
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