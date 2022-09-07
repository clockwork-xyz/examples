use {
    crate::state::*,
    anchor_lang::prelude::*,
    anchor_spl::{ 
        associated_token::AssociatedToken,
        token::{self, Mint, TokenAccount, Transfer}
    },
    clockwork_crank::state::{Queue, SEED_QUEUE, CrankResponse},
};

#[derive(Accounts)]
pub struct DisbursePayment<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(
        mut,
        associated_token::authority = payment,
        associated_token::mint = mint,
    )]
    pub escrow: Box<Account<'info, TokenAccount>>,

    #[account(address = payment.mint)]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [SEED_PAYMENT, payment.sender.key().as_ref(), payment.recipient.key().as_ref(), payment.mint.as_ref()],
        bump,
    )]
    pub payment: Box<Account<'info, Payment>>,

    #[account(
        signer, 
        seeds = [
            SEED_QUEUE, 
            payment.key().as_ref(), 
            "payment".as_bytes()
        ], 
        seeds::program = clockwork_crank::ID,
        bump,
    )]
    pub payment_queue: Box<Account<'info, Queue>>,

    #[account( 
        mut,
        associated_token::authority = payment.recipient,
        associated_token::mint = payment.mint,
    )]
    pub recipient_token_account: Box<Account<'info, TokenAccount>>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler(ctx: Context<'_, '_, '_, '_, DisbursePayment<'_>>) -> Result<CrankResponse> {
    // Get accounts
    let escrow = &ctx.accounts.escrow;
    let payment = &mut ctx.accounts.payment;
    let recipient_token_account = &ctx.accounts.recipient_token_account;
    let token_program = &ctx.accounts.token_program;

    // Update balance of payment account
    payment.balance = payment.balance.checked_sub(payment.disbursement_amount).unwrap();

    // Transfer from escrow to recipient's token account
    let bump = *ctx.bumps.get("payment").unwrap();
    token::transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(), 
            Transfer {
                from: escrow.to_account_info(),
                to: recipient_token_account.to_account_info(),
                authority: payment.to_account_info(),
            },             
            &[&[SEED_PAYMENT, payment.sender.as_ref(), payment.recipient.as_ref(), payment.mint.as_ref(), &[bump]]]),
        payment.disbursement_amount,
    )?;
        
    Ok(CrankResponse{ next_instruction: None })
}