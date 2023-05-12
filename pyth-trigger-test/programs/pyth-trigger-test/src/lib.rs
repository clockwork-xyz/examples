use std::str::FromStr;

use anchor_lang::{prelude::*, solana_program::system_program, InstructionData};
use clockwork_sdk::state::Thread;
use clockwork_sdk::utils::Equality;
use pyth_sdk_solana::{load_price_feed_from_account_info, PriceFeed};

declare_id!("3D9Z2VywLehXGJxMDBdt7gVhowTWHc2LNqtfRNcp2g5P");

#[program]
pub mod pyth_trigger_test {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let system_program = &ctx.accounts.system_program;
        let clockwork_program = &ctx.accounts.clockwork_program;
        let payer = &ctx.accounts.payer;
        let thread = &ctx.accounts.thread;
        let thread_authority = &ctx.accounts.thread_authority;

        // 1️⃣ Prepare an instruction to be automated.
        let price_pubkey =
            Pubkey::from_str("J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix").unwrap(); // SOL/USD (devnet)
        let target_ix = anchor_lang::solana_program::instruction::Instruction {
            program_id: ID,
            accounts: crate::accounts::Memo {
                thread: thread.key(),
                thread_authority: thread_authority.key(),
                price: price_pubkey,
            }
            .to_account_metas(Some(true)),
            data: crate::instruction::Memo {}.data(),
        };

        // 2️⃣ Define a trigger for the thread (Pyth feeds update multiple times per slot).
        // example price 2316687750
        let trigger = clockwork_sdk::state::Trigger::Pyth {
            price_feed: price_pubkey,
            equality: Equality::GreaterThanOrEqual,
            limit: 2310000000,
        };

        // 3️⃣ Create thread via CPI.
        let bump = *ctx.bumps.get("thread_authority").unwrap();
        clockwork_sdk::cpi::thread_create(
            CpiContext::new_with_signer(
                clockwork_program.to_account_info(),
                clockwork_sdk::cpi::ThreadCreate {
                    payer: payer.to_account_info(),
                    system_program: system_program.to_account_info(),
                    thread: thread.to_account_info(),
                    authority: thread_authority.to_account_info(),
                },
                &[&[THREAD_AUTHORITY_SEED, &[bump]]],
            ),
            anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL, // amount
            "".as_bytes().to_vec(),                                      // id
            vec![target_ix.into()],                                      // instructions
            trigger,                                                     // trigger
        )?;

        Ok(())
    }

    pub fn memo(ctx: Context<Memo>) -> Result<()> {
        let price_account_info = &ctx.accounts.price;
        let price_feed: PriceFeed = load_price_feed_from_account_info(&price_account_info).unwrap();
        msg!("{:?}", price_feed);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    /// The Clockwork thread program.
    #[account(address = clockwork_sdk::ID)]
    pub clockwork_program: Program<'info, clockwork_sdk::ThreadProgram>,

    /// The signer who will pay to initialize the program.
    /// (not to be confused with the thread executions).
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The Solana system program.
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    /// Address to assign to the newly created thread.
    #[account(mut, address = Thread::pubkey(thread_authority.key(), "".as_bytes().to_vec()))]
    pub thread: SystemAccount<'info>,

    /// The pda that will own and manage the thread.
    #[account(seeds = [THREAD_AUTHORITY_SEED], bump)]
    pub thread_authority: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct Memo<'info> {
    /// Verify that only this thread can execute the Increment Instruction
    #[account(signer, constraint = thread.authority.eq(&thread_authority.key()))]
    pub thread: Account<'info, Thread>,

    /// The Thread Admin
    /// The authority that was used as a seed to derive the thread address
    /// `thread_authority` should equal `thread.thread_authority`
    #[account(seeds = [THREAD_AUTHORITY_SEED], bump)]
    pub thread_authority: SystemAccount<'info>,

    /// CHECK: todo deserialize into price account
    #[account()]
    pub price: AccountInfo<'info>,
}

/// Seed for thread_authority PDA.
pub const THREAD_AUTHORITY_SEED: &[u8] = b"authority";
