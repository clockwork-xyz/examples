use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::{instruction::Instruction, system_program}},
    clockwork_sdk::queue_program::state::Trigger,
    std::mem::size_of,
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds = [SEED_AUTHORITY],
        bump,
        payer = signer,
        space = 8 + size_of::<Authority>(),
    )]
    pub authority: Account<'info, Authority>,

    #[account(address = clockwork_sdk::queue_program::ID)]
    pub clockwork: Program<'info, clockwork_sdk::queue_program::QueueProgram>,

    #[account(
        init,
        seeds = [SEED_EVENT],
        bump,
        payer = signer,
        space = 8 + size_of::<Event>(),
    )]
    pub event: Account<'info, Event>,

    #[account(
        seeds = [
            clockwork_sdk::queue_program::state::SEED_QUEUE, 
            authority.key().as_ref(), 
            "event".as_bytes()
        ], 
        seeds::program = clockwork_sdk::queue_program::ID,
        bump
    )]
    pub queue: SystemAccount<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    // Get accounts
    let authority = &ctx.accounts.authority;
    let clockwork = &ctx.accounts.clockwork;
    let event = &mut ctx.accounts.event;
    let queue = &ctx.accounts.queue;
    let signer = &ctx.accounts.signer;
    let system_program = &ctx.accounts.system_program;

    // Initialize event account
    event.timestamp = Clock::get().unwrap().unix_timestamp;
    event.user = signer.key();

    // Create a queue to process the events
    let bump = *ctx.bumps.get("authority").unwrap();
    let snapshot_kickoff_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority.key(), false),
            AccountMeta::new_readonly(event.key(), false),
            AccountMeta::new_readonly(queue.key(), true),
        ],
        data: clockwork_sdk::queue_program::utils::anchor_sighash("process_event").into(),
    };
    clockwork_sdk::queue_program::cpi::queue_create(
        CpiContext::new_with_signer(
            clockwork.to_account_info(),
            clockwork_sdk::queue_program::cpi::accounts::QueueCreate {
                authority: authority.to_account_info(),
                payer: signer.to_account_info(),
                queue: queue.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[&[SEED_AUTHORITY, &[bump]]]
        ),
        "event".into(),
        snapshot_kickoff_ix.into(),
        Trigger::Account { pubkey: event.key() }
    )?;

    Ok(())
}
