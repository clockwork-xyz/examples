use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        InstructionData,
        solana_program::{instruction::Instruction, native_token::LAMPORTS_PER_SOL, system_program},
    },
    clockwork_sdk::{
        self,
        state::{Thread, ThreadAccount, Trigger},
        ThreadProgram,
    },
    std::mem::size_of,
};

#[derive(Accounts)]
#[instruction(thread_id: Vec<u8>)]
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

    #[account(mut, address = Thread::pubkey(authority.key(), thread_id))]
    pub event_thread: SystemAccount<'info>,
}

pub fn handler(ctx: Context<Initialize>, thread_id: Vec<u8>) -> Result<()> {
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
        accounts: crate::accounts::ProcessEvent {
            authority: authority.key(),
            event: event.key(),
            event_thread: event_thread.key(),
        }.to_account_metas(Some(true)),
        data: crate::instruction::ProcessEvent {}.data(),
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
        LAMPORTS_PER_SOL,               // amount
        thread_id,                      // id
        vec![process_event_ix.into()],  // instructions
        Trigger::Account {              // trigger
            address: event.key(),
            offset: 0,
            size: 8,
        },
    )?;

    Ok(())
}


#[derive(Accounts)]
pub struct Reset<'info> {
    /// The signer.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The Clockwork thread program.
    #[account(address = clockwork_sdk::ID)]
    pub clockwork_program: Program<'info, clockwork_sdk::ThreadProgram>,

    /// The thread to reset.
    #[account(mut, address = thread.pubkey(), constraint = thread.authority.eq(&authority.key()))]
    pub thread: Account<'info, Thread>,

    /// Close the event account
    #[account(
        mut,
        seeds = [SEED_EVENT],
        bump,
        close = payer
    )]
    pub event: Account<'info, Event>,

    /// Close the authority account
    #[account(
        mut,
        seeds = [SEED_AUTHORITY],
        bump,
        close = payer
    )]
    pub authority: Account<'info, Authority>,

}

pub fn reset(ctx: Context<Reset>) -> Result<()> {
    // Get accounts
    let clockwork_program = &ctx.accounts.clockwork_program;
    let payer = &ctx.accounts.payer;
    let thread = &ctx.accounts.thread;
    let authority = &ctx.accounts.authority;

    // Delete thread via CPI.
    let bump = *ctx.bumps.get("authority").unwrap();
    clockwork_sdk::cpi::thread_delete(CpiContext::new_with_signer(
        clockwork_program.to_account_info(),
        clockwork_sdk::cpi::ThreadDelete {
            authority: authority.to_account_info(),
            close_to: payer.to_account_info(),
            thread: thread.to_account_info(),
        },
        &[&[SEED_AUTHORITY, &[bump]]],
    ))?;
    Ok(())
}
