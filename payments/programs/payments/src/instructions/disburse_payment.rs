use {
    crate::state::*,
    anchor_lang::prelude::*,
    anchor_spl::{ 
        associated_token::AssociatedToken,
        token::{self, Mint, TokenAccount, Transfer}
    },
    clockwork_scheduler::state::Queue,
};

#[derive(Accounts)]
pub struct DisbursePayment<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(
        mut,
        associated_token::authority = payment,
        associated_token::mint = payment.mint,
    )]
    pub escrow: Account<'info, TokenAccount>,

    #[account(address = payment.mint)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [SEED_PAYMENT, payment.sender.key().as_ref(), payment.recipient.key().as_ref(), payment.mint.as_ref()],
        bump,
        has_one = sender,
        has_one = recipient,
        has_one = mint,
    )]
    pub payment: Account<'info, Payment>,

    #[account(signer, constraint = queue.authority == payment.key())]
    pub queue: Account<'info, Queue>,

    #[account()]
    pub recipient: AccountInfo<'info>,

    #[account( 
        mut,
        associated_token::authority = payment.recipient,
        associated_token::mint = payment.mint,
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    #[account()]
    pub sender: AccountInfo<'info>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler(ctx: Context<'_, '_, '_, '_, DisbursePayment<'_>>) -> Result<()> {
    // Get accounts
    let escrow = &ctx.accounts.escrow;
    let payment = &mut ctx.accounts.payment;
    let recipient_token_account = &ctx.accounts.recipient_token_account;
    let token_program = &ctx.accounts.token_program;

    let bump = *ctx.bumps.get("payment").unwrap();

    // update balance of payment account
    payment.balance = payment.balance.checked_sub(payment.disbursement_amount).unwrap();

    // transfer from escrow to recipient's token account
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

    Ok(())
}
