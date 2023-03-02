use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{system_program, sysvar},
    },
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{self, Mint, TokenAccount},
    },
    std::mem::size_of,
};

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct CreatePayment<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut, 
        associated_token::authority = authority,
        associated_token::mint = mint,
    )]
    pub authority_token_account: Account<'info, TokenAccount>,

    #[account()]
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = authority,
        seeds = [
            SEED_PAYMENT, 
            authority.key().as_ref(), 
            mint.key().as_ref(),
            recipient.key().as_ref(), 
        ],
        bump,
        space = 8 + size_of::<Payment>(),
    )]
    pub payment: Account<'info, Payment>,

    /// CHECK: the recipient is validated by the seeds of the payment account
    #[account()]
    pub recipient: AccountInfo<'info>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, token::Token>,
}

pub fn handler<'info>(ctx: Context<CreatePayment>, amount: u64) -> Result<()> {
    // Get accounts.
    let authority = &ctx.accounts.authority;
    let authority_token_account = &mut ctx.accounts.authority_token_account;
    let mint = &ctx.accounts.mint;
    let payment = &mut ctx.accounts.payment;
    let recipient = &ctx.accounts.recipient;
    let token_program = &ctx.accounts.token_program;

    // Initialize the payment account.
    payment.new(
        amount,
        authority.key(),
        mint.key(),
        recipient.key(),
    )?;

    // Approve the payment pda to spend from the authority's token account.
    token::approve(
        CpiContext::new(
            token_program.to_account_info(), 
            token::Approve {
                authority: authority.to_account_info(),
                to: authority_token_account.to_account_info(),
                delegate: payment.to_account_info(), 
            }),
       u64::MAX
    )?;

    Ok(())
}
