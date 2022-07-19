use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::system_program},
    std::mem::size_of,
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds = [SEED_AUTHORITY, payer.key.as_ref()],
        bump,
        payer = payer,
        space = 8 + size_of::<Authority>(),
    )]
    pub authority: Account<'info, Authority>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = cronos_scheduler::ID)]
    pub scheduler_program: Program<'info, cronos_scheduler::program::CronosScheduler>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, Initialize<'info>>) -> Result<()> {
    // Get accounts
    let authority = &mut ctx.accounts.authority;
    let payer = &mut ctx.accounts.payer;
    let scheduler_program = &ctx.accounts.scheduler_program;
    let system_program = &ctx.accounts.system_program;

    // Get remaining accounts
    let manager = ctx.remaining_accounts.get(0).unwrap();

    // initialize authority account
    authority.new(manager.key(), payer.key())?;

    // get authority bump
    let bump = *ctx.bumps.get("authority").unwrap();

    // create manager
    cronos_scheduler::cpi::manager_new(CpiContext::new_with_signer(
        scheduler_program.to_account_info(),
        cronos_scheduler::cpi::accounts::ManagerNew {
            authority: authority.to_account_info(),
            manager: manager.to_account_info(),
            payer: payer.to_account_info(),
            system_program: system_program.to_account_info(),
        },
        &[&[SEED_AUTHORITY, authority.payer.as_ref(), &[bump]]],
    ))?;

    Ok(())
}
