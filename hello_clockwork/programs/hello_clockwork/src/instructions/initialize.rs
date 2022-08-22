use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{native_token::LAMPORTS_PER_SOL, system_program, instruction::Instruction},
    },
    clockwork_crank::{
        program::ClockworkCrank,
        state::{Trigger, SEED_QUEUE},
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

    #[account(address = clockwork_crank::ID)]
    pub clockwork_program: Program<'info, ClockworkCrank>,

    #[account(
        seeds = [
            SEED_QUEUE, 
            authority.key().as_ref(), 
            "hello".as_bytes()
        ], 
        seeds::program = clockwork_crank::ID,
        bump
     )]
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
    let hello_queue = &ctx.accounts.hello_queue;
    let clockwork_program = &ctx.accounts.clockwork_program;
    let system_program = &ctx.accounts.system_program;

    // get authorit bump
    let bump = *ctx.bumps.get("authority").unwrap();

    // define ix
    let hello_clowckwork_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![ 
            AccountMeta::new_readonly(authority.key(), false),
            AccountMeta::new_readonly(hello_queue.key(), true)
        ],
        data: clockwork_crank::anchor::sighash("hello_world").to_vec(),
    };

    // initialize queue
    clockwork_crank::cpi::queue_create(
        CpiContext::new_with_signer(
            clockwork_program.to_account_info(),
            clockwork_crank::cpi::accounts::QueueCreate {
                authority: authority.to_account_info(),
                payer: payer.to_account_info(),
                queue: hello_queue.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[&[SEED_AUTHORITY, &[bump]]],
        ),
        LAMPORTS_PER_SOL,
        hello_clowckwork_ix.into(),
        "hello".into(),
        Trigger::Cron {
            schedule: "*/15 * * * * * *".into(),
        },
    )?;

    Ok(())
}
