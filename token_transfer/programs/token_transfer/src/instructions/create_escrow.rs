use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::system_program},
    anchor_spl::token::Mint,
    clockwork_scheduler::{program::ClockworkScheduler, state::{Queue, SEED_QUEUE}},
    std::mem::size_of,
};

#[derive(Accounts)]
#[instruction(amount: u64, transfer_rate: u64)]
pub struct CreateEscrow<'info> {
    #[account(
        seeds = [SEED_AUTHORITY],
        bump,
    )]
    pub authority: Account<'info, Authority>,

    #[account(
        init,
        payer = sender,
        seeds = [SEED_ESCROW, sender.key().as_ref(), recipient.key().as_ref()],
        bump,
        space = 8 + size_of::<Escrow>(),
    )]
    pub escrow: Account<'info, Escrow>,

    pub mint: Account<'info, Mint>,

    #[account(
      seeds = [SEED_QUEUE, authority.key().as_ref(), "token_transfer_queue".as_ref()], 
      seeds::program = clockwork_scheduler::ID, 
      bump,
    )]
    pub queue: Account<'info, Queue>,

    #[account()]
    pub recipient: AccountInfo<'info>,

    #[account(address = clockwork_scheduler::ID)]
    pub scheduler_program: Program<'info, ClockworkScheduler>,

    #[account(mut)]
    pub sender: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<CreateEscrow>, amount: u64, transfer_rate: u64) -> Result<()> {
    // Get accounts
    let escrow = &mut ctx.accounts.escrow;
    let mint = &ctx.accounts.mint;
    let queue = &ctx.accounts.queue;
    let recipient = &ctx.accounts.recipient;
    let sender = &mut ctx.accounts.sender;

    // initialize Escrow
    escrow.new(
        sender.key(),
        recipient.key(),
        mint.key(),
        amount,
        transfer_rate,
        Some(queue.key()),
    )?;

    Ok(())
}
