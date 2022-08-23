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
    
    #[account(address = clockwork_crank::ID)]
    pub clockwork_program: Program<'info, ClockworkCrank>,
    
    #[account(
        init,
        seeds = [SEED_CRANK],
        bump,
        payer = payer,
        space = 8 + size_of::<Crank>(),
    )]
    pub crank: Account<'info, Crank>,

    #[account(
        seeds = [
            SEED_QUEUE, 
            crank.key().as_ref(), 
            "crank".as_bytes()
        ], 
        seeds::program = clockwork_crank::ID,
        bump
     )]
    pub crank_queue: SystemAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>, 

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, Initialize<'info>>) -> Result<()> {
    // Get accounts
    let clockwork_program = &ctx.accounts.clockwork_program;
    let crank = &mut ctx.accounts.crank;
    let crank_queue = &ctx.accounts.crank_queue;
    let payer = &ctx.accounts.payer;
    let system_program = &ctx.accounts.system_program;

    // Get extra accounts
    let event_q = ctx.remaining_accounts.get(0).unwrap();

    // initialize crank account
    crank.new()?;

    // get authorit bump
    let bump = *ctx.bumps.get("authority").unwrap();

    // define ix
    let crank_events_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![ 
            AccountMeta::new_readonly(crank.key(), false),
            AccountMeta::new_readonly(crank_queue.key(), true),
            // Extra Accounts
            AccountMeta::new_readonly(event_q.key(), false)
        ],
        data: clockwork_crank::anchor::sighash("read_events").to_vec(),
    };

    // initialize queue
    clockwork_crank::cpi::queue_create(
        CpiContext::new_with_signer(
            clockwork_program.to_account_info(),
            clockwork_crank::cpi::accounts::QueueCreate {
                authority: crank.to_account_info(),
                payer: payer.to_account_info(),
                queue: crank_queue.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[&[SEED_CRANK, &[bump]]],
        ),
        LAMPORTS_PER_SOL,
        crank_events_ix.into(),
        "crank".into(),
        Trigger::Cron {
            schedule: "*/15 * * * * * *".into(),
        },
    )?;

    Ok(())
}