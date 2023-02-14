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
    amount: u64,
    other_amount_threshold: u64,
    sqrt_price_limit: u128,
    amount_specified_is_input: bool,
    a_to_b: bool,
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

    /// CHECK: 
    pub whirlpool: AccountInfo<'info>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, DcaCreate<'info>>,
    amount: u64,
    other_amount_threshold: u64,
    sqrt_price_limit: u128,
    amount_specified_is_input: bool,
    a_to_b: bool,
) -> Result<()> {
    // get accounts
    let authority = &ctx.accounts.authority;
    let authority_a_vault = &mut ctx.accounts.authority_a_vault;
    let authority_b_vault = &mut ctx.accounts.authority_b_vault;
    let dca = &mut ctx.accounts.dca;
    let a_mint = &ctx.accounts.a_mint;
    let b_mint = &ctx.accounts.b_mint;
    let token_program = &ctx.accounts.token_program;
    let whirlpool = &ctx.accounts.whirlpool;

    // initialize dca account
    dca.new(
        authority.key(),
        whirlpool.key(),
        a_mint.key(),
        b_mint.key(),
        amount,
        other_amount_threshold,
        sqrt_price_limit,
        amount_specified_is_input,
        a_to_b
    )?;

    // approve the dca account to spend from the authority's token account.
    token::approve(
        CpiContext::new(
            token_program.to_account_info(),
            token::Approve {
                to: if dca.a_to_b { authority_a_vault.to_account_info() } else { authority_b_vault.to_account_info() },
                delegate: dca.to_account_info(),
                authority: authority.to_account_info(),
            },
        ),
        u64::MAX,
    )?;

    Ok(())
}
