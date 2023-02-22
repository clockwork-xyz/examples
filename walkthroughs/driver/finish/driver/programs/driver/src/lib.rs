mod id;
use id::ID;
use std::str::FromStr;
use std::mem::size_of;
use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, system_program},
};

use clockwork_sdk::{
    self,
    state::{Thread, Trigger},
    ThreadProgram,
};


/// Seed for thread_authority pda ⚠️ make sure it matches whatever you are using on
/// client-side
pub const THREAD_AUTHORITY_SEED: &[u8] = b"authority";
/// The Program that the Thread will talk to
pub const HELLO_CLOCKWORK_PROGRAM_ID: &str = "Gj5eCcUsKAPAJMftoKK8bLKCSByTaWANQzrESeWZT5Qe";


#[program]
pub mod driver {
    use super::*;

    pub fn create_thread(ctx: Context<CreateThread>, thread_label: String) -> Result<()> {
        // 1️⃣ Prepare an instruction to feed to the Thread
        let target_ix = Instruction {
            program_id: Pubkey::from_str(HELLO_CLOCKWORK_PROGRAM_ID).unwrap(),
            accounts: vec![],
            data: clockwork_sdk::utils::anchor_sighash("hello_ix").into(),
        };

        // 2️⃣ Define a trigger for the Thread to execute
        let trigger = Trigger::Cron {
            schedule: "*/10 * * * * * *".into(),
            skippable: true,
        };

        // Get accounts
        let system_program = &ctx.accounts.system_program;
        let clockwork_program = &ctx.accounts.clockwork_program;
        let payer = &ctx.accounts.payer;
        let thread = &ctx.accounts.thread;
        let thread_authority = &ctx.accounts.thread_authority;

        // 3️⃣ Create Thread
        let bump = *ctx.bumps.get("thread_authority").unwrap();
        // Accounts Meta Infos:
        // https://docs.rs/clockwork-thread-program/1.4.2/src/clockwork_thread_program/instructions/thread_create.rs.html#9
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
        //           "isSigner": false 👈 signing will be handled by cpi anyway
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
            cpi_ctx,
            thread_label,
            target_ix.into(),
            trigger,
        )?;

        Ok(())
    }
}

/// Debug Signer Seeds, according to what's passed on from the client
fn debug_signer_seeds(seeds: &[&[u8]]) {
    let thread_authority_pda = Pubkey::create_program_address(seeds, &crate::ID).unwrap();
    // const [threadAuthority] = PublicKey.findProgramAddressSync(
    //     [anchor.utils.bytes.utf8.encode("authority")],
    //     program.programId
    // );
    // console.log("threadAuthority: ", threadAuthority);
    //
    // In the front-end the above code should result into this 👇 (at least for my program id)
    let expected_thread_authority_pda = "FJYGDyuFe292z8oQzcVxD7JBBiACmASv8YwbnbdhqbPu";
    assert_eq!(thread_authority_pda.to_string(), expected_thread_authority_pda);
}


#[derive(Accounts)]
#[instruction(thread_label: String)]
pub struct CreateThread<'info> {
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    /// Clockwork Program (Thread Program)
    #[account(address = clockwork_sdk::ID)]
    pub clockwork_program: Program<'info, ThreadProgram>,

    /// Who's paying
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Address to assign to the newly created Thread
    #[account(mut, address = Thread::pubkey(thread_authority.key(), thread_label))]
    pub thread: SystemAccount<'info>,

    /// Thread Admin, not signer but it will be use to pseudo-sign by the driver program
    #[account(
    seeds = [THREAD_AUTHORITY_SEED],
    bump,
    )]
    pub thread_authority: Account<'info, Authority>,

    // #[account(
    // init,
    // seeds = [THREAD_AUTHORITY_SEED],
    // bump,
    // payer = payer,
    // space = 8 + size_of::<Authority>(),
    // )]
    // pub thread_authority: Account<'info, Authority>,
}

#[account]
#[derive(Debug)]
pub struct Authority {}

impl Authority {
    pub fn pubkey() -> Pubkey {
        Pubkey::find_program_address(&[THREAD_AUTHORITY_SEED], &crate::ID).0
    }
}

impl TryFrom<Vec<u8>> for Authority {
    type Error = Error;
    fn try_from(data: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Authority::try_deserialize(&mut data.as_slice())
    }
}
