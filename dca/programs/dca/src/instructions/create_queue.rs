use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{
            native_token::LAMPORTS_PER_SOL, system_program,
        },
    },
    clockwork_scheduler::{state::SEED_QUEUE, program::ClockworkScheduler},
    std::mem::size_of,
};

#[derive(Accounts)]
pub struct CreateQueue<'info> {
    #[account(
        init,
        seeds = [SEED_AUTHORITY, payer.key().as_ref()],
        bump,
        payer = payer,
        space = 8 + size_of::<Authority>(),
    )]
    pub authority: Account<'info, Authority>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [SEED_QUEUE, authority.key().as_ref(), "dca_queue".as_bytes()], 
        seeds::program = clockwork_scheduler::ID, 
        bump,
	)]
    pub queue: SystemAccount<'info>,

    #[account(address = clockwork_scheduler::ID)]
    pub scheduler_program: Program<'info, ClockworkScheduler>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, CreateQueue<'info>>) -> Result<()> {
    // Get accounts
    let authority = &mut ctx.accounts.authority;
    let payer = &ctx.accounts.payer;
    let queue = &ctx.accounts.queue;
    let scheduler_program = &ctx.accounts.scheduler_program;
    let system_program = &ctx.accounts.system_program;

    // get authority bump
    let bump = *ctx.bumps.get("authority").unwrap();

    // initialize authority account
    authority.new(payer.key())?;

    // Create queue
    clockwork_scheduler::cpi::queue_new(
        CpiContext::new_with_signer(
            scheduler_program.to_account_info(),
            clockwork_scheduler::cpi::accounts::QueueNew {
                authority: authority.to_account_info(),
                payer: payer.to_account_info(),
                queue: queue.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[&[SEED_AUTHORITY, authority.payer.as_ref(), &[bump]]],
        ),
        LAMPORTS_PER_SOL,
        "dca_queue".to_string(),
        "*/15 * * * * * *".into(),
    )?;

    Ok(())
}
