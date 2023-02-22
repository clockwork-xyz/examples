use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{instruction::Instruction, system_program},
    },
    clockwork_sdk::{
        self,
        state::{Thread, Trigger},
        ThreadProgram,
    },
    std::mem::size_of,
};

#[derive(Accounts)]
#[instruction(thread_label: String)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds = [SEED_AUTHORITY],
        bump,
        payer = signer,
        space = 8 + size_of::<Authority>(),
    )]
    pub authority: Account<'info, Authority>,

    #[account(address = clockwork_sdk::ID)]
    pub clockwork: Program<'info, ThreadProgram>,

    #[account(
        init,
        seeds = [SEED_EVENT],
        bump,
        payer = signer,
        space = 8 + size_of::<Event>(),
    )]
    pub event: Account<'info, Event>,

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(mut, address = Thread::pubkey(authority.key(), thread_label))]
    pub event_thread: SystemAccount<'info>,
}

pub fn handler(ctx: Context<Initialize>, thread_label: String) -> Result<()> {
    // Get accounts
    let authority = &ctx.accounts.authority;
    let clockwork = &ctx.accounts.clockwork;
    let event = &mut ctx.accounts.event;
    let event_thread = &ctx.accounts.event_thread;
    let signer = &ctx.accounts.signer;
    let system_program = &ctx.accounts.system_program;

    // Initialize event account
    event.timestamp = Clock::get().unwrap().unix_timestamp;
    event.user = signer.key();

    // Create a thread to process the events
    let bump = *ctx.bumps.get("authority").unwrap();
    let process_event_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority.key(), false),
            AccountMeta::new_readonly(event.key(), false),
            AccountMeta::new_readonly(event_thread.key(), true),
        ],
        data: clockwork_sdk::utils::anchor_sighash("process_event").into(),
    };

    clockwork_sdk::cpi::thread_create(
        CpiContext::new_with_signer(
            clockwork.to_account_info(),
            clockwork_sdk::cpi::ThreadCreate {
                authority: authority.to_account_info(),
                payer: signer.to_account_info(),
                system_program: system_program.to_account_info(),
                thread: event_thread.to_account_info(),
            },
            &[&[SEED_AUTHORITY, &[bump]]],
        ),
        thread_label,
        process_event_ix.into(),
        Trigger::Account {
            address: event.key(),
            offset: 0,
            size: 8,
        },
    )?;

    Ok(())
}
