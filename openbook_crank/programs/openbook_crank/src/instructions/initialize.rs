use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::system_program},
    anchor_spl::dex::serum_dex::state::Market,
    std::mem::size_of,
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = signer,
        seeds = [
            SEED_CRANK, 
            signer.key().as_ref(), 
            market.key().as_ref(), 
        ],
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

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, Initialize<'info>>,
) -> Result<()> {
    // Get accounts
    let crank = &mut ctx.accounts.crank;
    let dex_program = &ctx.accounts.dex_program;
    let event_queue = &ctx.accounts.event_queue;
    let market = &ctx.accounts.market;
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
        market.key(),
        10,
    )?;

    Ok(())
}
