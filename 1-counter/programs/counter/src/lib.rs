pub mod id;

pub use id::ID;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::Instruction, native_token::LAMPORTS_PER_SOL, system_program,
};
use anchor_lang::InstructionData;
use clockwork_sdk::state::{ThreadAccount};


#[program]
pub mod counter {
    use super::*;

    // Just here to delete the `Counter` account and reset the tests to a clean state
    pub fn delete(_ctx: Context<Delete>) -> Result<()> {
        Ok(())
    }

    pub fn increment(ctx: Context<Increment>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;

        counter.current_value = counter.current_value.checked_add(1).unwrap();
        counter.updated_at = Clock::get().unwrap().unix_timestamp;
        msg!("Counter value: {}, updated_at: {}", counter.current_value, counter.updated_at);
        Ok(())
    }

    pub fn initialize(ctx: Context<Initialize>, thread_id: Vec<u8>) -> Result<()> {
        // Get accounts
        let system_program = &ctx.accounts.system_program;
        let clockwork_program = &ctx.accounts.clockwork_program;
        let payer = &ctx.accounts.payer;
        let thread = &ctx.accounts.thread;
        let thread_authority_pda = &ctx.accounts.thread_authority_pda;
        let bump = *ctx.bumps.get("thread_authority_pda").unwrap();
        let counter = &mut ctx.accounts.counter;

        // 1️⃣ Prepare an instruction to feed to the Thread
        let target_ix = Instruction {
            program_id: ID,
            accounts: crate::accounts::Increment {
                system_program: system_program.key(),
                counter: counter.key(),
                thread: thread.key(),
                thread_authority_pda: thread_authority_pda.key(),
            }.to_account_metas(Some(true)),
            data: crate::instruction::Increment {}.data(),
        };

        // 2️⃣ Define a trigger for the Thread to execute
        let trigger = clockwork_sdk::state::Trigger::Cron {
            schedule: "*/10 * * * * * *".into(),
            skippable: true,
        };

        // 3️⃣ Create Thread via CPI
        clockwork_sdk::cpi::thread_create(
            CpiContext::new_with_signer(
                clockwork_program.to_account_info(),
                clockwork_sdk::cpi::ThreadCreate {
                    payer: payer.to_account_info(),
                    system_program: system_program.to_account_info(),
                    thread: thread.to_account_info(),
                    authority: thread_authority_pda.to_account_info(),
                },
                &[&[THREAD_AUTHORITY_SEED, &[bump]]],
            ),
            LAMPORTS_PER_SOL,   // amount
            thread_id,              // id
            vec![target_ix.into()], // Instructions vec
            trigger,                // Trigger
        )?;

        Ok(())
    }

    pub fn delete_thread(ctx: Context<DeleteThread>) -> Result<()> {
        // Get accounts
        let clockwork_program = &ctx.accounts.clockwork_program;
        let payer = &ctx.accounts.payer;
        let thread = &ctx.accounts.thread;
        let thread_authority_pda = &ctx.accounts.thread_authority_pda;
        let bump = *ctx.bumps.get("thread_authority_pda").unwrap();

        // The actual CPI
        clockwork_sdk::cpi::thread_delete(CpiContext::new_with_signer(
            clockwork_program.to_account_info(),
            clockwork_sdk::cpi::ThreadDelete {
                authority: thread_authority_pda.to_account_info(),
                close_to: payer.to_account_info(),
                thread: thread.to_account_info(),
            },
            &[&[THREAD_AUTHORITY_SEED, &[bump]]],
        ))?;
        Ok(())
    }
}

/*
**  Accounts
 */
#[account]
#[derive(Debug)]
pub struct Counter {
    pub current_value: u64,
    pub updated_at: i64,
}

/*
**  Validation Structs
 */

/// Seed for `Counter` account Program Derived Address
/// ⚠️ Make sure it matches whatever you are using on the client-side
pub const SEED_COUNTER: &[u8] = b"counter";

/// Seed for thread_authority pda
/// ⚠️ Make sure it matches whatever you are using on the client-side
pub const THREAD_AUTHORITY_SEED: &[u8] = b"authority";

#[derive(Accounts)]
pub struct Increment<'info> {
    /// The system program.
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    /// The counter account.
    #[account(
    mut,
    seeds = [SEED_COUNTER],
    bump,
    )]
    pub counter: Account<'info, Counter>,

    /// Verify that only this thread can execute the Increment Instruction
    #[account(
    signer,
    constraint = thread.authority.eq(& thread_authority_pda.key())
    )]
    pub thread: Account<'info, clockwork_sdk::state::Thread>,

    /// The Thread Admin
    /// The authority that was used as a seed to derive the thread address
    /// `thread_authority_pda` should equal `thread.thread_authority`
    #[account(
    seeds = [THREAD_AUTHORITY_SEED],
    bump,
    )]
    pub thread_authority_pda: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct Delete<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
    mut,
    seeds = [SEED_COUNTER],
    bump,
    close = payer
    )]
    pub counter: Account<'info, Counter>,
}

#[derive(Accounts)]
#[instruction(thread_id: Vec < u8 >)]
pub struct Initialize<'info> {
    /// Who's paying for this Initialize Transaction
    /// (not to be confused with the thread executions)
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    /// Clockwork Program (Thread Program)
    #[account(address = clockwork_sdk::ID)]
    pub clockwork_program: Program<'info, clockwork_sdk::ThreadProgram>,

    /// Address to assign to the newly created Thread
    #[account(
    mut,
    address = clockwork_sdk::state::Thread::pubkey(thread_authority_pda.key(), thread_id)
    )]
    pub thread: SystemAccount<'info>,

    /// The Thread Admin
    /// The address that was used as a seed to derive the thread address with
    /// `clockworkProvider.getThreadPDA(threadAuthority, threadId)`
    /// `thread_authority_pda` should equal `thread.authority`
    #[account(
    seeds = [THREAD_AUTHORITY_SEED],
    bump,
    )]
    pub thread_authority_pda: SystemAccount<'info>,

    #[account(
    init,
    payer = payer,
    seeds = [SEED_COUNTER],
    bump,
    space = 8 + std::mem::size_of::< Counter > (),
    )]
    pub counter: Account<'info, Counter>,
}

#[derive(Accounts)]
pub struct DeleteThread<'info> {
    /// Who's paying
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Clockwork Program (Thread Program)
    #[account(address = clockwork_sdk::ID)]
    pub clockwork_program: Program<'info, clockwork_sdk::ThreadProgram>,

    /// Address to assign to the newly created Thread
    #[account(
    mut,
    address = thread.pubkey(),
    constraint = thread.authority.eq(& thread_authority_pda.key())
    )]
    pub thread: Account<'info, clockwork_sdk::state::Thread>,

    /// The Thread Admin
    /// The address that was used as a seed to derive the thread address
    /// `thread_authority_pda` should equal `thread.authority`
    #[account(
    seeds = [THREAD_AUTHORITY_SEED],
    bump,
    )]
    pub thread_authority_pda: SystemAccount<'info>,
}
