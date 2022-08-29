use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{system_program, sysvar},
    },
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{self, Mint, TokenAccount, Transfer},
    },
};

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct TopUpPayment<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(
        mut, 
        associated_token::authority = payment,
        associated_token::mint = mint,
    )]
    pub escrow: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [SEED_PAYMENT, payment.sender.key().as_ref(), payment.recipient.key().as_ref(), payment.mint.as_ref()],
        bump,
        has_one = recipient,
        has_one = sender,
        has_one = mint
    )]
    pub payment: Account<'info, Payment>,

    pub mint: Account<'info, Mint>,

    /// CHECK: the recipient is validated by the payment account
    #[account()]
    pub recipient: AccountInfo<'info>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    /// CHECK: the sender is validated by the payment account
    #[account(mut)]
    pub sender: Signer<'info>,

    #[account(
        mut,
        associated_token::authority = payment.sender,
        associated_token::mint = payment.mint,
    )]
    pub sender_token_account: Account<'info, TokenAccount>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, TopUpPayment<'info>>,
    amount: u64,
) -> Result<()> {
    // Get accounts
    let payment = &mut ctx.accounts.payment;
    let sender = &mut ctx.accounts.sender;
    let sender_token_account = &ctx.accounts.sender_token_account;
    let token_program = &ctx.accounts.token_program;
    let escrow = &mut ctx.accounts.escrow;

    // update balance of payment account
    payment.balance = payment.balance.checked_add(amount).unwrap();

    // deposit funds from sender's token account to escrow token account
    token::transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: sender_token_account.to_account_info().clone(),
                to: escrow.to_account_info().clone(),
                authority: sender.to_account_info().clone(),
            },
        ),
        amount,
    )?;

    Ok(())
}
