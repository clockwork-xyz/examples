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

    pub fn create_thread(ctx: Context<CreateThread>, thread_id: Vec<u8>) -> Result<()> {
        // Get accounts
        let system_program = &ctx.accounts.system_program;
        let clockwork_program = &ctx.accounts.clockwork_program;
        let payer = &ctx.accounts.payer;
        let thread = &ctx.accounts.thread;
        let thread_authority = &ctx.accounts.thread_authority;
        let bump = *ctx.bumps.get("thread_authority").unwrap();
        let counter = &mut ctx.accounts.counter;

        // ⚠️ Set the authority for the counter
        if counter.updated_at == 0 {
            counter.authority = thread_authority.key();
        }

        // 1️⃣ Prepare an instruction to feed to the Thread
        let target_ix = Instruction {
            program_id: ID,
            accounts: crate::accounts::Increment {
                system_program: system_program.key(),
                payer: clockwork_sdk::utils::PAYER_PUBKEY,
                counter: counter.key(),
                thread: thread.key(),
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
                    authority: thread_authority.to_account_info(),
                    payer: payer.to_account_info(),
                    system_program: system_program.to_account_info(),
                    thread: thread.to_account_info(),
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
        let thread_authority = &ctx.accounts.thread_authority;
        let bump = *ctx.bumps.get("thread_authority").unwrap();

        // The actual CPI
        clockwork_sdk::cpi::thread_delete(CpiContext::new_with_signer(
            clockwork_program.to_account_info(),
            clockwork_sdk::cpi::ThreadDelete {
                authority: thread_authority.to_account_info(),
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
    pub authority: Pubkey,
    pub current_value: u64,
    pub updated_at: i64,
}

/*
**  Validation Structs
 */

/// Seed for `Counter` account Program Derived Address
/// ⚠️make sure it matches whatever you are using on the client-side
pub const SEED_COUNTER: &[u8] = b"counter";

#[derive(Accounts)]
pub struct Increment<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(
    mut,
    seeds = [SEED_COUNTER],
    bump,
    )]
    pub counter: Account<'info, Counter>,

    /// Verify that only this thread can execute the Increment Instruction
    #[account(
    signer,
    address = thread.pubkey(),
    constraint = thread.authority == counter.authority
    )]
    pub thread: Account<'info, clockwork_sdk::state::Thread>,
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

/// Seed for thread_authority pda
/// ⚠️ Make sure it matches whatever you are using on the client-side
pub const THREAD_AUTHORITY_SEED: &[u8] = b"authority";

#[derive(Accounts)]
#[instruction(thread_id: Vec < u8 >)]
pub struct CreateThread<'info> {
    /// Who's paying
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
    address = clockwork_sdk::state::Thread::pubkey(thread_authority.key(), thread_id)
    )]
    pub thread: SystemAccount<'info>,

    /// Thread Admin
    #[account(
    seeds = [THREAD_AUTHORITY_SEED],
    bump,
    )]
    pub thread_authority: SystemAccount<'info>,

    #[account(
    init_if_needed,
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
    )]
    pub thread: Account<'info, clockwork_sdk::state::Thread>,

    /// Thread Admin
    #[account(
    seeds = [THREAD_AUTHORITY_SEED],
    bump,
    )]
    pub thread_authority: SystemAccount<'info>,
}
