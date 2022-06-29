use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::system_program},
    anchor_spl::dex::serum_dex::state::{Market, OpenOrders},
    std::mem::size_of,
};

#[derive(Accounts)]
pub struct InitOrder<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(constraint = fund.manager == buyer.key())]
    pub fund: Account<'info, Fund>,

    #[account(
      init
      seeds = [
        SEED_ORDER, 
        fund.key().as_ref(),
        buyer.key().as_ref()
      ]
      bump,
      payer = buyer
    )]
    pub order: Account<'info, Order>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<InitOrder>, amount: u64) -> Result<()> {
    // Get accounts
    let buyer = &ctx.accounts.buyer;
    let fund = &ctx.accounts.fund;
    let order = &mut ctx.accounts.order;

    // initialize order account
    order.new(
   fund.key(),
  buyer.key(),
        amount,
                Side::Bid,
          fund.index_token_mint,
           0,
            0
      )?;

    Ok(())
}
