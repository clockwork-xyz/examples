use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::system_program,
    },
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{self, Mint, TokenAccount},
    },
    std::mem::size_of,
};

#[derive(Accounts)]
#[instruction(
    amount_in: u64,
    minimum_amount_out: u64,
)]
pub struct DcaCreate<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        associated_token::authority = authority,
        associated_token::mint = a_mint,
    )]
    pub authority_a_vault: Box<Account<'info, TokenAccount>>,

    /// CHECK:
    pub a_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::authority = authority,
        associated_token::mint = b_mint,
    )]
    pub authority_b_vault: Box<Account<'info, TokenAccount>>,
    
    /// CHECK:
    pub b_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        seeds = [
            SEED_DCA, 
            authority.key().as_ref(), 
            a_mint.key().as_ref(),
            b_mint.key().as_ref()
        ],
        bump,
        payer = authority,
        space = 8 + size_of::<Dca>(),
    )]
    pub dca: Box<Account<'info, Dca>>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = a_mint,
        associated_token::authority = dca
    )]
    pub dca_a_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = b_mint,
        associated_token::authority = dca
    )]
    pub dca_b_vault: Box<Account<'info, TokenAccount>>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, DcaCreate<'info>>,
    amount_in: u64,
    minimum_amount_out: u64,
) -> Result<()> {
    // Get accounts
    let authority = &ctx.accounts.authority;
    let authority_a_vault = &mut ctx.accounts.authority_a_vault;
    let a_mint = &ctx.accounts.a_mint;
    let b_mint = &ctx.accounts.b_mint;
    let dca = &mut ctx.accounts.dca;
    let token_program = &ctx.accounts.token_program;

    // initialize dca account
    dca.new(
        authority.key(),
        a_mint.key(),
        b_mint.key(),
        amount_in,
        minimum_amount_out,
    )?;

    // Approve the dca account to spend from the authority's token account.
    token::approve(
        CpiContext::new(
            token_program.to_account_info(),
            token::Approve {
                to: authority_a_vault.to_account_info(),
                delegate: dca.to_account_info(),
                authority: authority.to_account_info(),
            },
        ),
        u64::MAX,
    )?;

    Ok(())
}
