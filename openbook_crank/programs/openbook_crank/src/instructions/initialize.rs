use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::system_program},
    anchor_spl::{
        dex::serum_dex::state::Market,
        token::{Mint, TokenAccount},
    },
    std::mem::size_of,
};

#[derive(Accounts)]
#[instruction(id: String)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = signer,
        seeds = [SEED_CRANK, signer.key().as_ref(), market.key().as_ref(), id.as_bytes()],
        bump,
        space = 8 + size_of::<Crank>(),
    )]
    pub crank: Box<Account<'info, Crank>>,

    pub dex_program: Program<'info, OpenBookDex>,

    /// CHECK: this account is manually verified in handler
    #[account()]
    pub event_queue: AccountInfo<'info>,

    /// CHECK: this account is manually verified in handler
    #[account()]
    pub market: AccountInfo<'info>,

    #[account()]
    pub mint_a: Box<Account<'info, Mint>>,

    #[account(
        constraint = mint_a_vault.mint == mint_a.key()
    )]
    pub mint_a_vault: Box<Account<'info, TokenAccount>>,

    #[account()]
    pub mint_b: Box<Account<'info, Mint>>,

    #[account(
        constraint = mint_b_vault.mint == mint_b.key()
    )]
    pub mint_b_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, Initialize<'info>>,
    id: String,
) -> Result<()> {
    // Get accounts
    let crank = &mut ctx.accounts.crank;
    let dex_program = &ctx.accounts.dex_program;
    let event_queue = &ctx.accounts.event_queue;
    let market = &ctx.accounts.market;
    let mint_a_vault = &ctx.accounts.mint_a_vault;
    let mint_b_vault = &ctx.accounts.mint_b_vault;
    let signer = &ctx.accounts.signer;

    // validate market
    let market_data = Market::load(market, &dex_program.key()).unwrap();
    let val = unsafe { std::ptr::addr_of!(market_data.event_q).read_unaligned() };
    let market_event_queue = Pubkey::new(safe_transmute::to_bytes::transmute_one_to_bytes(
        core::convert::identity(&val),
    ));

    require_keys_eq!(event_queue.key(), market_event_queue);

    // initialize crank account
    crank.new(
        signer.key(),
        event_queue.key(),
        id,
        10,
        market.key(),
        mint_a_vault.key(),
        mint_b_vault.key(),
    )?;

    Ok(())
}
