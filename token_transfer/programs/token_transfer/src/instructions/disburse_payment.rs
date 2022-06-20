use {
    crate::state::*,
    anchor_lang::prelude::*,
    anchor_spl::{ 
        associated_token::AssociatedToken,
        token::{self, Mint, TokenAccount, Transfer}
    },
    cronos_scheduler::state::Manager,
};

#[derive(Accounts)]
pub struct DisbursePayment<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(seeds = [SEED_AUTHORITY], bump, has_one = manager)]
    pub authority: Box<Account<'info, Authority>>,

    #[account(
        seeds = [SEED_ESCROW, sender.key().as_ref(), recipient.key().as_ref()],
        bump,
        has_one = sender,
        has_one = recipient,
        has_one = mint,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(signer, has_one = authority)]
    pub manager: Account<'info, Manager>,

    #[account(address = escrow.mint)]
    pub mint: Account<'info, Mint>,

    #[account()]
    pub recipient: AccountInfo<'info>,

    #[account( 
        mut,
        associated_token::authority = recipient,
        associated_token::mint = mint,
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    #[account()]
    pub sender: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::authority = escrow,
        associated_token::mint = mint,
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler(ctx: Context<'_, '_, '_, '_, DisbursePayment<'_>>) -> Result<()> {
    // Get accounts
    let escrow = &ctx.accounts.escrow;
    let vault = &ctx.accounts.vault;
    let recipient_token_account = &ctx.accounts.recipient_token_account;
    let token_program = &ctx.accounts.token_program;

    let bump = *ctx.bumps.get("escrow").unwrap();

    // transfer from vault to recipient's token account
    token::transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(), 
            Transfer {
                from: vault.to_account_info().clone(),
                to: recipient_token_account.to_account_info().clone(),
                authority: escrow.to_account_info().clone(),
            }, &[&[SEED_ESCROW, &escrow.sender.to_bytes(), &escrow.recipient.to_bytes(), &[bump]]]),
        escrow.transfer_rate,
    )?;

    Ok(())
}
