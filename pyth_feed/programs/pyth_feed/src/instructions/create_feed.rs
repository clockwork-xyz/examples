use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{instruction::Instruction, system_program},
    },
    clockwork_sdk::{
        ID as thread_program_ID,
        cpi::{
            thread_create, thread_update,
            ThreadCreate, ThreadUpdate,
        },
        state::{Trigger, Thread, ThreadSettings},
        ThreadProgram,
    },
    std::mem::size_of,
};
#[derive(Accounts)]
pub struct CreateFeed<'info> {
    #[account(address = thread_program_ID)]
    pub clockwork: Program<'info, ThreadProgram>,

    #[account(
        init,
        seeds = [SEED_FEED, signer.key().as_ref()],
        bump,
        payer = signer,
        space = 8 + size_of::<Feed>()
    )]
    pub feed: Account<'info, Feed>,

    #[account(address = Thread::pubkey(feed.key(), "feed".into()))]
    pub thread: SystemAccount<'info>,

    /// CHECK: this account should be a pyth feed account
    pub pyth_price_feed: AccountInfo<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, CreateFeed<'info>>) -> Result<()> {
    // Get accounts
    let clockwork = &ctx.accounts.clockwork;
    let feed = &mut ctx.accounts.feed;
    let thread = &ctx.accounts.thread;
    let pyth_price_feed = &ctx.accounts.pyth_price_feed;
    let signer = &ctx.accounts.signer;
    let system_program = &ctx.accounts.system_program;

    // initialize PDA feed account
    feed.new(signer.key(), pyth_price_feed.key())?;

    // get feed bump
    let bump = *ctx.bumps.get("feed").unwrap();

    // build process feed ix
    let proceess_feed_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(feed.key(), false),
            AccountMeta::new_readonly(feed.pyth_price_feed, false),
            AccountMeta::new(thread.key(), true),
        ],
        data: clockwork_sdk::utils::anchor_sighash("process_feed").into(),
    };

    // initialize thread
    thread_create(
        CpiContext::new_with_signer(
            clockwork.to_account_info(),
            ThreadCreate {
                authority: feed.to_account_info(),
                payer: signer.to_account_info(),
                thread: thread.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[&[SEED_FEED, feed.authority.as_ref(), &[bump]]],
        ),
        "feed".into(),
        proceess_feed_ix.into(),
        Trigger::Account {
            address: feed.pyth_price_feed,
            offset: 4 + 8 + 8 + 4 + 4 + 4 + 4,
            size: 8,
        },
    )?;

    // set the rate limit of the thread to crank 1 time per slot
    thread_update(
        CpiContext::new_with_signer(
            clockwork.to_account_info(),
            ThreadUpdate {
                authority: feed.to_account_info(),
                thread: thread.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[&[SEED_FEED, feed.authority.as_ref(), &[bump]]],
        ),
        ThreadSettings {
            fee: None,
            kickoff_instruction: None,
            rate_limit: Some(1),
            trigger: None,
        },
    )?;

    Ok(())
}
