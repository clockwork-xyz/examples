use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::{system_program, sysvar}},
    anchor_spl::{ 
        associated_token::AssociatedToken,
        token::{self, Mint, TokenAccount, Transfer}
    },
    clockwork_sdk::{queue_program::accounts::{Queue, QueueAccount}, CrankResponse},
};

#[derive(Accounts)]
pub struct DisbursePayment<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(address = payment.mint)]
    pub mint: Box<Account<'info, Mint>>,
    
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SEED_PAYMENT, 
            payment.sender.as_ref(), 
            payment.recipient.as_ref(), 
            payment.mint.as_ref()
        ],
        bump,
        has_one = sender,
        has_one = recipient,
        has_one = mint,
    )]
    pub payment: Box<Account<'info, Payment>>,

    #[account(
        signer, 
        address = payment_queue.pubkey(),
        constraint = payment_queue.authority.eq(&payment.sender),
    )]
    pub payment_queue: Box<Account<'info, Queue>>,

    /// CHECK: the recipient is validated by the payment account
    #[account()]
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

    /// CHECK: the sender is validated by the payment account
    #[account()]
    pub sender: AccountInfo<'info>,

    #[account(
        mut,
        token::authority = payment,
        token::mint = mint,
    )]
    pub sender_token_account: Account<'info, TokenAccount>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler(ctx: Context<'_, '_, '_, '_, DisbursePayment<'_>>) -> Result<CrankResponse> {
    // Get accounts
    let payment = &mut ctx.accounts.payment;
    let recipient_token_account = &ctx.accounts.recipient_token_account;
    let sender_token_account = &mut ctx.accounts.sender_token_account;
    let token_program = &ctx.accounts.token_program;

    // get payment bump
    let bump = *ctx.bumps.get("payment").unwrap();

    // transfer from sender's ATA to recipient's ATA
    token::transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(), 
            Transfer {
                from: sender_token_account.to_account_info(),
                to: recipient_token_account.to_account_info(),
                authority: payment.to_account_info(),
            },             
            &[&[SEED_PAYMENT, payment.sender.as_ref(), payment.recipient.as_ref(), payment.mint.as_ref(), &[bump]]]),
        payment.amount,
    )?;

    Ok(CrankResponse{ next_instruction: None, kickoff_instruction: None })
}
