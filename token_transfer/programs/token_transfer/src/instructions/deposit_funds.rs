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
pub struct DepositFunds<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(
        mut,
        seeds = [SEED_ESCROW, sender.key().as_ref(), recipient.key().as_ref()],
        bump,
    )]
    pub escrow: Account<'info, Escrow>,

    pub mint: Account<'info, Mint>,

    #[account()]
    pub recipient: AccountInfo<'info>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = clockwork_scheduler::ID)]
    pub scheduler_program: Program<'info, clockwork_scheduler::program::ClockworkScheduler>,

    #[account(mut)]
    pub sender: Signer<'info>,

    #[account(
        mut,
        associated_token::authority = escrow.sender,
        associated_token::mint = escrow.mint,
    )]
    pub sender_token_account: Account<'info, TokenAccount>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,

    #[account(
        init,
        payer = sender,
        associated_token::authority = escrow,
        associated_token::mint = mint,
    )]
    pub vault: Account<'info, TokenAccount>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, DepositFunds<'info>>) -> Result<()> {
    // Get accounts
    let escrow = &mut ctx.accounts.escrow;
    let sender = &mut ctx.accounts.sender;
    let sender_token_account = &ctx.accounts.sender_token_account;
    let token_program = &ctx.accounts.token_program;
    let vault = &ctx.accounts.vault;

    // deposit funds from sender's token account to vault token account
    token::transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: sender_token_account.to_account_info().clone(),
                to: vault.to_account_info().clone(),
                authority: sender.to_account_info().clone(),
            },
        ),
        escrow.amount,
    )?;

    Ok(())
}
