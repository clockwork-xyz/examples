use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{instruction::Instruction, system_program, sysvar},
    },
    std::mem::size_of,
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        seeds = [SEED_AUTHORITY],
        bump,
        payer = payer,
        space = 8 + size_of::<Authority>(),
    )]
    pub authority: Account<'info, Authority>,

    #[account(address = sysvar::clock::ID)]
    pub clock: Sysvar<'info, Clock>,

    #[account(address = cronos_scheduler::ID)]
    pub scheduler_program: Program<'info, cronos_scheduler::program::CronosScheduler>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, Initialize<'info>>) -> Result<()> {
    // Get accounts
    let payer = &ctx.accounts.payer;
    let authority = &mut ctx.accounts.authority;
    let clock = &ctx.accounts.clock;
    let scheduler_program = &ctx.accounts.scheduler_program;
    let system_program = &ctx.accounts.system_program;

    // Get remaining Accounts
    let message_fee = ctx.remaining_accounts.get(0).unwrap();
    let message_queue = ctx.remaining_accounts.get(1).unwrap();
    let manager = ctx.remaining_accounts.get(2).unwrap();
    let message_task = ctx.remaining_accounts.get(3).unwrap();

    // Initialize Accounts
    authority.new(manager.key())?;

    // Create Manager
    let bump = *ctx.bumps.get("authority").unwrap();
    cronos_scheduler::cpi::manager_new(CpiContext::new_with_signer(
        scheduler_program.to_account_info(),
        cronos_scheduler::cpi::accounts::ManagerNew {
            authority: authority.to_account_info(),
            manager: manager.to_account_info(),
            payer: payer.to_account_info(),
            system_program: system_program.to_account_info(),
        },
        &[&[SEED_AUTHORITY, &[bump]]],
    ))?;

    // Create queue
    cronos_scheduler::cpi::queue_new(
        CpiContext::new_with_signer(
            scheduler_program.to_account_info(),
            cronos_scheduler::cpi::accounts::QueueNew {
                authority: authority.to_account_info(),
                clock: clock.to_account_info(),
                fee: message_fee.to_account_info(),
                manager: manager.to_account_info(),
                payer: payer.to_account_info(),
                queue: message_queue.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[&[SEED_AUTHORITY, &[bump]]],
        ),
        "*/15 * * * * * *".into(),
    )?;

    // create ix
    let message_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority.key(), false),
            AccountMeta::new_readonly(manager.key(), true),
            AccountMeta::new_readonly(sysvar::clock::ID, false),
        ],
        data: sighash("global", "hello_world").into(),
    };

    // Create task with the hello world ix and add it to the queue
    cronos_scheduler::cpi::task_new(
        CpiContext::new_with_signer(
            scheduler_program.to_account_info(),
            cronos_scheduler::cpi::accounts::TaskNew {
                authority: authority.to_account_info(),
                manager: manager.to_account_info(),
                payer: payer.to_account_info(),
                queue: message_queue.to_account_info(),
                system_program: system_program.to_account_info(),
                task: message_task.to_account_info(),
            },
            &[&[SEED_AUTHORITY, &[bump]]],
        ),
        vec![message_ix.into()],
    )?;

    Ok(())
}

fn sighash(namespace: &str, name: &str) -> [u8; 8] {
    let preimage = format!("{}:{}", namespace, name);
    let mut sighash = [0u8; 8];
    sighash.copy_from_slice(
        &anchor_lang::solana_program::hash::hash(preimage.as_bytes()).to_bytes()[..8],
    );
    sighash
}
