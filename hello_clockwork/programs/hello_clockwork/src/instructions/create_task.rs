use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{
            instruction::Instruction, system_program, sysvar,
        },
    },
    clockwork_scheduler::state::{SEED_TASK, SEED_QUEUE, Queue},
};

#[derive(Accounts)]
pub struct CreateTask<'info> {
    #[account(
        seeds = [SEED_AUTHORITY],
        bump,
    )]
    pub authority: Account<'info, Authority>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
      seeds = [SEED_QUEUE, authority.key().as_ref(), "hello_queue".as_ref()], 
      seeds::program = clockwork_scheduler::ID, 
      bump,
    )]
    pub queue: Account<'info, Queue>,

    #[account(address = clockwork_scheduler::ID)]
    pub scheduler_program: Program<'info, clockwork_scheduler::program::ClockworkScheduler>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(
			seeds = [SEED_TASK, queue.key().as_ref(), (0 as u64).to_be_bytes().as_ref()], 
			seeds::program = clockwork_scheduler::ID, 
			bump
		)]
	pub task: SystemAccount<'info>,

}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, CreateTask<'info>>) -> Result<()> {
    // Get accounts
    let authority = &mut ctx.accounts.authority;
    let payer = &ctx.accounts.payer;
    let queue = &ctx.accounts.queue;
    let scheduler_program = &ctx.accounts.scheduler_program;
    let system_program = &ctx.accounts.system_program;
    let task = &ctx.accounts.task;

    // get authorit bump
    let bump = *ctx.bumps.get("authority").unwrap();

    // create ix
    let hello_world_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority.key(), false),
            AccountMeta::new_readonly(queue.key(), true),
            AccountMeta::new_readonly(sysvar::clock::ID, false),
        ],
        data: clockwork_scheduler::anchor::sighash("hello_world").to_vec(),
    };

    // Create task with the hello world ix and add it to the queue
    clockwork_scheduler::cpi::task_new(
        CpiContext::new_with_signer(
            scheduler_program.to_account_info(),
            clockwork_scheduler::cpi::accounts::TaskNew {
                authority: authority.to_account_info(),
                payer: payer.to_account_info(),
                queue: queue.to_account_info(),
                system_program: system_program.to_account_info(),
                task: task.to_account_info(),
            },
            &[&[SEED_AUTHORITY, &[bump]]],
        ),
        vec![hello_world_ix.into()],
    )?;

    Ok(())
}
