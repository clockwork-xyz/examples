use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{instruction::Instruction, system_program},
    },
    clockwork_sdk::queue_program::{
        self,
        accounts::{Queue, Trigger},
    },
    std::mem::size_of,
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds = [SEED_AUTHORITY],
        bump,
        payer = payer,
        space = 8 + size_of::<Authority>(),
    )]
    pub authority: Account<'info, Authority>,

    #[account(address = queue_program::ID)]
    pub clockwork_program: Program<'info, clockwork_sdk::queue_program::QueueProgram>,

    #[account(address = Queue::pubkey(authority.key(), "hello".into()))]
    pub hello_queue: SystemAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, Initialize<'info>>) -> Result<()> {
    // Get accounts
    let authority = &mut ctx.accounts.authority;
    let payer = &ctx.accounts.payer;
    let hello_queue = &mut ctx.accounts.hello_queue;
    let clockwork_program = &ctx.accounts.clockwork_program;
    let system_program = &ctx.accounts.system_program;

    // define ix
    let hello_clockwork_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority.key(), false),
            AccountMeta::new_readonly(hello_queue.key(), true),
        ],
        data: clockwork_sdk::queue_program::utils::anchor_sighash("hello_world").to_vec(),
    };

    // initialize queue
    let bump = *ctx.bumps.get("authority").unwrap();
    clockwork_sdk::queue_program::cpi::queue_create(
        CpiContext::new_with_signer(
            clockwork_program.to_account_info(),
            clockwork_sdk::queue_program::cpi::accounts::QueueCreate {
                authority: authority.to_account_info(),
                payer: payer.to_account_info(),
                queue: hello_queue.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[&[SEED_AUTHORITY, &[bump]]],
        ),
        "hello".into(),
        hello_clockwork_ix.into(),
        Trigger::Cron {
            schedule: "*/15 * * * * * *".into(),
            skippable: true,
        },
    )?;

    Ok(())
}
