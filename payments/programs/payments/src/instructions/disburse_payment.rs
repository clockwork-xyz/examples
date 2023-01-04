use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::{system_program, sysvar}},
    anchor_spl::{ 
        associated_token::AssociatedToken,
        token::{self, Mint, TokenAccount, Transfer}
    },
    clockwork_sdk::{
        state::{Thread, ThreadAccount, ThreadResponse},
    },
};

#[derive(Accounts)]
pub struct DisbursePayment<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// CHECK: The authority is validated by the payment account
    #[account(address = payment.authority)]
    pub authority: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::authority = authority,
        associated_token::mint = mint,
    )]
    pub authority_token_account: Account<'info, TokenAccount>,

    #[account(address = payment.mint)]
    pub mint: Box<Account<'info, Mint>>,
    
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SEED_PAYMENT, 
            payment.authority.as_ref(), 
            payment.mint.as_ref(),
            payment.recipient.as_ref(), 
        ],
        bump,
        has_one = authority,
        has_one = mint,
        has_one = recipient,
    )]
    pub payment: Box<Account<'info, Payment>>,

    #[account(
        signer, 
        address = thread.pubkey(),
        constraint = thread.authority.eq(&payment.authority),
    )]
    pub thread: Box<Account<'info, Thread>>,

    /// CHECK: The recipient is validated by the payment account
    #[account(address = payment.recipient)]
    pub recipient: AccountInfo<'info>,

    #[account( 
        init_if_needed,
        payer = payer,
        associated_token::authority = recipient,
        associated_token::mint = mint,
    )]
    pub recipient_token_account: Box<Account<'info, TokenAccount>>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler(ctx: Context<DisbursePayment>) -> Result<ThreadResponse> {
    // Get accounts.
    let authority_token_account = &mut ctx.accounts.authority_token_account;
    let payment = &mut ctx.accounts.payment;
    let recipient_token_account = &ctx.accounts.recipient_token_account;
    let token_program = &ctx.accounts.token_program;

    // Transfer tokens from authority's ATA to recipient's ATA.
    let bump = *ctx.bumps.get("payment").unwrap();
    token::transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(), 
            Transfer {
                from: authority_token_account.to_account_info(),
                to: recipient_token_account.to_account_info(),
                authority: payment.to_account_info(),
            },             
            &[&[
                SEED_PAYMENT, 
                payment.authority.as_ref(),
                payment.mint.as_ref(),  
                payment.recipient.as_ref(),
                &[bump]]
            ]),
        payment.amount,
    )?;

    Ok(ThreadResponse::default())
}
