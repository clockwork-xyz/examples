use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{instruction::Instruction, system_program},
    },
    clockwork_sdk::queue_program::{
        self,
        accounts::{Queue, Trigger},
        QueueProgram,
    },
    std::mem::size_of,
};
#[derive(Accounts)]
#[instruction(data_feed: Pubkey)]
pub struct CreateFeed<'info> {
    #[account(address = queue_program::ID)]
    pub clockwork: Program<'info, QueueProgram>,

    #[account(
        init,
        seeds = [SEED_FEED],
        bump,
        payer = signer,
        space = 8 + size_of::<Feed>()
    )]
    pub feed: Account<'info, Feed>,

    #[account(address = Queue::pubkey(feed.key(), "feed".into()))]
    pub queue: SystemAccount<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, CreateFeed<'info>>,
    pyth_feed: Pubkey,
) -> Result<()> {
    let clockwork = &ctx.accounts.clockwork;
    let feed = &mut ctx.accounts.feed;
    let queue = &ctx.accounts.queue;
    let signer = &ctx.accounts.signer;
    let system_program = &ctx.accounts.system_program;

    feed.new(feed.key(), pyth_feed)?;

    let bump = *ctx.bumps.get("feed").unwrap();
    let proceess_feed_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(feed.key(), false),
            AccountMeta::new_readonly(feed.price_feed, false),
            AccountMeta::new(queue.key(), true),
        ],
        data: clockwork_sdk::anchor_sighash("process_pyth_feed").into(),
    };

    clockwork_sdk::queue_program::cpi::queue_create(
        CpiContext::new_with_signer(
            clockwork.to_account_info(),
            clockwork_sdk::queue_program::cpi::accounts::QueueCreate {
                authority: feed.to_account_info(),
                payer: signer.to_account_info(),
                queue: queue.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[&[SEED_FEED, &[bump]]],
        ),
        "feed".into(),
        proceess_feed_ix.into(),
        Trigger::Account { pubkey: pyth_feed },
    )?;

    Ok(())
}
