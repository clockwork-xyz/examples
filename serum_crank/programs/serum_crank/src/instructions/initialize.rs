use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{instruction::Instruction, system_program},
    },
    anchor_spl::{
        dex::serum_dex::state::Market,
        token::{Mint, TokenAccount},
    },
    clockwork_sdk::{
        queue_program::{
            self,
            accounts::{Queue, Trigger},
            QueueProgram,
        },
        PAYER_PUBKEY,
    },
    std::mem::size_of,
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(address = queue_program::ID)]
    pub clockwork_program: Program<'info, QueueProgram>,

    #[account(
        init,
        payer = payer,
        seeds = [SEED_CRANK, market.key().as_ref()],
        bump,
        space = 8 + size_of::<Crank>(),
    )]
    pub crank: Account<'info, Crank>,

    #[account(address = Queue::pubkey(crank.key(), "crank".into()))]
    pub crank_queue: SystemAccount<'info>,

    #[account(address = anchor_spl::dex::ID)]
    pub dex_program: Program<'info, anchor_spl::dex::Dex>,

    /// CHECK: this account is manually verified in handler
    #[account()]
    pub event_queue: AccountInfo<'info>,

    /// CHECK: this account is manually verified in handler
    #[account()]
    pub market: AccountInfo<'info>,

    #[account()]
    pub mint_a: Account<'info, Mint>,

    #[account(
        constraint = mint_a_vault.mint == mint_a.key()
    )]
    pub mint_a_vault: Account<'info, TokenAccount>,

    #[account()]
    pub mint_b: Account<'info, Mint>,

    #[account(
        constraint = mint_b_vault.mint == mint_b.key()
    )]
    pub mint_b_vault: Account<'info, TokenAccount>,

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
    let event_queue = &ctx.accounts.event_queue;
    let market = &ctx.accounts.market;
    let mint_a_vault = &ctx.accounts.mint_a_vault;
    let mint_b_vault = &ctx.accounts.mint_b_vault;
    let payer = &ctx.accounts.payer;
    let system_program = &ctx.accounts.system_program;

    // validate market
    let market_data = Market::load(market, &dex_program.key()).unwrap();
    let val = unsafe { std::ptr::addr_of!(market_data.event_q).read_unaligned() };
    let market_event_queue = Pubkey::new(safe_transmute::to_bytes::transmute_one_to_bytes(
        core::convert::identity(&val),
    ));
    require_keys_eq!(event_queue.key(), market_event_queue);

    // initialize crank account
    crank.new(
        market.key(),
        event_queue.key(),
        mint_a_vault.key(),
        mint_b_vault.key(),
        10,
    )?;

    // get authorit bump
    let bump = *ctx.bumps.get("crank").unwrap();

    // define ix
    let read_events_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(crank.key(), false),
            AccountMeta::new(crank_queue.key(), true),
            AccountMeta::new_readonly(dex_program.key(), false),
            AccountMeta::new_readonly(event_queue.key(), false),
            AccountMeta::new_readonly(market.key(), false),
            AccountMeta::new_readonly(mint_a_vault.key(), false),
            AccountMeta::new_readonly(mint_b_vault.key(), false),
            AccountMeta::new(PAYER_PUBKEY, true),
            AccountMeta::new_readonly(system_program.key(), false),
        ],
        data: clockwork_sdk::anchor_sighash("read_events").to_vec(),
    };

    // initialize queue
    clockwork_sdk::queue_program::cpi::queue_create(
        CpiContext::new_with_signer(
            clockwork_program.to_account_info(),
            clockwork_sdk::queue_program::cpi::accounts::QueueCreate {
                authority: crank.to_account_info(),
                payer: payer.to_account_info(),
                queue: crank_queue.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[&[SEED_CRANK, crank.market.as_ref(), &[bump]]],
        ),
        "crank".into(),
        read_events_ix.into(),
        Trigger::Cron {
            schedule: "*/15 * * * * * *".into(),
            skippable: true,
        },
    )?;

    Ok(())
}
