use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{system_program, instruction::Instruction},
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
        space = 8 + size_of::<Crank>() + 2048,
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

    #[account(address = anchor_spl::dex::ID)]
    pub dex_program: Program<'info, anchor_spl::dex::Dex>,

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
    let dex_program = &ctx.accounts.dex_program;
    let payer = &ctx.accounts.payer;
    let system_program = &ctx.accounts.system_program;

    // Get extra accounts
    let market = ctx.remaining_accounts.get(0).unwrap();
    let mint_a_vault = ctx.remaining_accounts.get(1).unwrap();
    let mint_b_vault = ctx.remaining_accounts.get(2).unwrap();
    let event_queue = ctx.remaining_accounts.get(3).unwrap();
    // initialize crank account
    crank.new()?;

    // get authorit bump
    let bump = *ctx.bumps.get("crank").unwrap();

    // define ix
    let read_events_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![ 
            AccountMeta::new(crank.key(), false),
            AccountMeta::new_readonly(crank_queue.key(), true),
            AccountMeta::new_readonly(dex_program.key(), false),
            AccountMeta::new_readonly(system_program.key(), false),
            // Extra Accounts
            AccountMeta::new(market.key(), false),
            AccountMeta::new(mint_a_vault.key(), false),
            AccountMeta::new(mint_b_vault.key(), false),
            AccountMeta::new(event_queue.key(), false)
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
        read_events_ix.into(),
        "crank".into(),
        Trigger::Cron {
            schedule: "*/15 * * * * * *".into(),
        },
    )?;

    Ok(())
}