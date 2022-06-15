use {
    crate::state::*,
    anchor_lang::prelude::*,
    anchor_spl::token::{self, TokenAccount, Transfer},
    cronos_scheduler::state::Manager,
};

#[derive(Accounts)]
pub struct DisbursePayment<'info> {
    #[account(seeds = [SEED_AUTHORITY], bump, has_one = manager)]
    pub authority: Box<Account<'info, Authority>>,

    #[account(
        mut,
        constraint = escrow.payer == sender.key(),
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(signer, constraint = manager.authority == authority.key())]
    pub manager: Account<'info, Manager>,

    pub receiver: AccountInfo<'info>,

    #[account(mut, constraint = receiver_token_account.owner == receiver.key())]
    pub receiver_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub sender: AccountInfo<'info>,

    #[account(
        mut,
        constraint = vault.owner == escrow.key()
    )]
    pub vault: Account<'info, TokenAccount>,

    pub token_program: AccountInfo<'info>,
}

pub fn handler(ctx: Context<'_, '_, '_, '_, DisbursePayment<'_>>) -> Result<()> {
    // Get accounts
    let escrow = &mut ctx.accounts.escrow;
    let recipient_token_account = &mut ctx.accounts.receiver_token_account;
    let token_program = &ctx.accounts.token_program;
    let vault = &mut ctx.accounts.vault;

    // transfer from vault to receiver's token account
    token::transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: vault.to_account_info().clone(),
                to: recipient_token_account.to_account_info().clone(),
                authority: escrow.to_account_info().clone(),
            },
        ),
        escrow.transfer_rate,
    )?;

    Ok(())
}
