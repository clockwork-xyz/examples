use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::system_program},
    anchor_spl::token::Mint,
    std::mem::size_of,
};

#[derive(Accounts)]
pub struct Create<'info> {
    #[account(
        init,
        payer = sender,
        seeds = [SEED_ESCROW, sender.key().as_ref(), recipient.key().as_ref()],
        bump,
        space = 8 + size_of::<Escrow>(),
    )]
    pub escrow: Account<'info, Escrow>,

    pub mint: Box<Account<'info, Mint>>,

    #[account()]
    pub recipient: AccountInfo<'info>,

    #[account(address = cronos_scheduler::ID)]
    pub scheduler_program: Program<'info, cronos_scheduler::program::CronosScheduler>,

    #[account(mut)]
    pub sender: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<Create>) -> Result<()> {
    // Get accounts
    let escrow = &mut ctx.accounts.escrow;
    let mint = &ctx.accounts.mint;
    let recipient = &ctx.accounts.recipient;
    let sender = &mut ctx.accounts.sender;

    // Get remaining Accounts
    let disburse_queue = ctx.remaining_accounts.get(0).unwrap();

    // initialize Escrow
    escrow.new(
        mint.key(),
        recipient.key(),
        sender.key(),
        Some(disburse_queue.key()),
        0,
        0,
    )?;

    Ok(())
}
