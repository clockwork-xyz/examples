pub mod id;

pub use id::ID;
use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use anchor_lang::solana_program::{instruction::Instruction, system_program};
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
use clockwork_sdk::state::{ThreadAccount};

/// Seed for `Counter` account Program Derived Address
/// ⚠️make sure it matches whatever you are using on the client-side
pub const SEED_COUNTER: &[u8] = b"counter";

/// Seed for thread_authority pda
/// ⚠️make sure it matches whatever you are using on the client-side
pub const THREAD_AUTHORITY_SEED: &[u8] = b"authority";

#[program]
pub mod counter {
    use super::*;

    pub fn reset(ctx: Context<Increment>) -> Result<()> {
        msg!(
            "Previous Counter value: {}, updated_at: {}",
            ctx.accounts.counter.current_value,
            ctx.accounts.counter.updated_at
        );
        ctx.accounts.counter.current_value = 0;
        ctx.accounts.counter.updated_at = 0;
        msg!(
            "New Counter value: {}, updated_at: {}",
            ctx.accounts.counter.current_value,
            ctx.accounts.counter.updated_at
        );
        Ok(())
    }


    pub fn increment(ctx: Context<Increment>) -> Result<clockwork_sdk::state::ThreadResponse> {
        ctx.accounts.counter.current_value = ctx.accounts.counter.current_value.checked_add(1).unwrap();
        ctx.accounts.counter.updated_at = Clock::get().unwrap().unix_timestamp;

        msg!(
            "Counter value: {}, updated_at: {}",
            ctx.accounts.counter.current_value,
            ctx.accounts.counter.updated_at
        );

        Ok(clockwork_sdk::state::ThreadResponse::default())
    }

    pub fn create_thread(ctx: Context<CreateThread>, thread_id: Vec<u8>) -> Result<()> {
        // Get accounts
        let system_program = &ctx.accounts.system_program;
        let clockwork_program = &ctx.accounts.clockwork_program;
        let payer = &ctx.accounts.payer;
        let thread = &ctx.accounts.thread;
        let thread_authority = &ctx.accounts.thread_authority;
        let counter = &ctx.accounts.counter;

        // 1️⃣ Prepare an instruction to feed to the Thread
        let target_ix = Instruction {
            program_id: ID,
            accounts: crate::accounts::Increment {
                system_program: system_program.key(),
                payer: clockwork_sdk::utils::PAYER_PUBKEY,
                counter: counter.key(),
            }.to_account_metas(Some(true)),
            data: crate::instruction::Increment {}.data(),
        };

        // 2️⃣ Define a trigger for the Thread to execute
        let trigger = clockwork_sdk::state::Trigger::Cron {
            schedule: "*/10 * * * * * *".into(),
            skippable: true,
        };

        // 3️⃣ Create Thread
        let bump = *ctx.bumps.get("thread_authority").unwrap();
        // Accounts Meta Infos:
        // https://docs.rs/clockwork-thread-program/latest/src/clockwork_thread_program/instructions/thread_create.rs.html#9
        //         {
        //           "name": "payer",
        //           "isMut": true,
        //           "isSigner": true
        //         },
        //         {
        //           "name": "thread",
        //           "isMut": true,
        //           "isSigner": false
        //         },
        //         {
        //           "name": "authority",
        //           "isMut": false,
        //           "isSigner": true 👈 signing will be handled by cpi anyway
        //         }

        // ThreadCreate CPI Context
        let seeds = &[THREAD_AUTHORITY_SEED, &[bump]];
        // debug_signer_seeds(seeds);
        let signer = [&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            clockwork_program.to_account_info(),
            clockwork_sdk::cpi::ThreadCreate {
                authority: thread_authority.to_account_info(),
                payer: payer.to_account_info(),
                system_program: system_program.to_account_info(),
                thread: thread.to_account_info(),
            },
            &signer,
        );

        // The actual CPI
        clockwork_sdk::cpi::thread_create(
            cpi_ctx,                    // CpiContext
            2 * LAMPORTS_PER_SOL,       // amount
            thread_id,                  // id
            vec![target_ix.into()],     // Instructions vec
            trigger,                    // Trigger
        )?;

        Ok(())
    }

    pub fn delete_thread(ctx: Context<DeleteThread>) -> Result<()> {
        // Get accounts
        let clockwork_program = &ctx.accounts.clockwork_program;
        let payer = &ctx.accounts.payer;
        let thread = &ctx.accounts.thread;
        let thread_authority = &ctx.accounts.thread_authority;
        let bump = *ctx.bumps.get("thread_authority").unwrap();

        // The actual CPI
        clockwork_sdk::cpi::thread_delete(
            CpiContext::new_with_signer(
                clockwork_program.to_account_info(),
                clockwork_sdk::cpi::ThreadDelete {
                    authority: thread_authority.to_account_info(),
                    close_to: payer.to_account_info(),
                    thread: thread.to_account_info(),
                },
                &[&[THREAD_AUTHORITY_SEED, &[bump]]],
            )
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Increment<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
    init_if_needed,
    payer = payer,
    seeds = [SEED_COUNTER],
    bump,
    space = 8 + std::mem::size_of::< Counter > (),
    )]
    pub counter: Account<'info, Counter>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(Debug)]
pub struct Counter {
    pub current_value: u64,
    pub updated_at: i64,
}

#[derive(Accounts)]
#[instruction(thread_id: Vec < u8 >)]
pub struct CreateThread<'info> {
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    /// Clockwork Program (Thread Program)
    #[account(address = clockwork_sdk::ID)]
    pub clockwork_program: Program<'info, clockwork_sdk::ThreadProgram>,

    /// Who's paying
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Address to assign to the newly created Thread
    #[account(
    mut,
    address = clockwork_sdk::state::Thread::pubkey(thread_authority.key(), thread_id))]
    pub thread: SystemAccount<'info>,

    /// Thread Admin, not signer but it will be use to pseudo-sign by the driver program
    #[account(
    seeds = [THREAD_AUTHORITY_SEED],
    bump,
    )]
    pub thread_authority: SystemAccount<'info>,

    #[account(
    mut,
    seeds = [SEED_COUNTER],
    bump,
    )]
    pub counter: Account<'info, Counter>,
}

#[derive(Accounts)]
pub struct DeleteThread<'info> {
    /// Clockwork Program (Thread Program)
    #[account(address = clockwork_sdk::ID)]
    pub clockwork_program: Program<'info, clockwork_sdk::ThreadProgram>,

    /// Who's paying
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Address to assign to the newly created Thread
    #[account(
    mut,
    address = thread.pubkey(),
    )]
    pub thread: Account<'info, clockwork_sdk::state::Thread>,

    /// Thread Admin, not signer but it will be use to pseudo-sign by the driver program
    #[account(
    seeds = [THREAD_AUTHORITY_SEED],
    bump,
    )]
    pub thread_authority: SystemAccount<'info>,
}
