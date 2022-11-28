use {
    crate::objects::*,
    anchor_lang::{prelude::*, solana_program::system_program, system_program::{transfer, Transfer}},
    clockwork_sdk::thread_program::accounts::{Thread, ThreadAccount},
    pyth_sdk_solana::load_price_feed_from_account_info,
};

#[derive(Accounts)]
pub struct Calc<'info> {
    #[account(
        mut,
        seeds = [
            SEED_STAT, 
            stat.price_feed.as_ref(), 
            stat.authority.as_ref(),
            stat.id.as_bytes(),
        ],
        bump,
    )]
    pub stat: Account<'info, Stat>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: this account is manually being checked against the stat account's price_feed field
    #[account(
        constraint = price_feed.key() == stat.price_feed
    )]
    pub price_feed: AccountInfo<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(
        constraint = thread.authority == stat.authority,
        address = thread.pubkey(),
        signer
    )]
    pub thread: Account<'info, Thread>,
}

pub fn handler<'info>(ctx: Context<Calc<'info>>) -> Result<()> {
    let payer = &mut ctx.accounts.payer;
    let price_feed = &ctx.accounts.price_feed;
    let stat = &mut ctx.accounts.stat;
    let system_program = &ctx.accounts.system_program;

    match load_price_feed_from_account_info(&price_feed.to_account_info()) {
        Ok(price_feed) => {
            // get price unchecked
            let price = price_feed.get_price_unchecked();
            // calculate time weighted average
            stat.twap(price.publish_time, price.price)?;

            // realloc account size
            let new_len = 8 + stat.try_to_vec()?.len();
            stat.to_account_info().realloc(new_len, false)?;

            let minimum_rent = Rent::get().unwrap().minimum_balance(new_len);

            if minimum_rent > stat.to_account_info().lamports() {
                transfer(
                    CpiContext::new(
                        system_program.to_account_info(),
                        Transfer {
                            from: payer.to_account_info(),
                            to: stat.to_account_info(),
                        },            
                    ),
                    minimum_rent.checked_sub(stat.to_account_info().lamports()).unwrap()
                )?;
            }

            msg!(
                "TWA Price: {} for lookback window: {}",
                stat.twap,
                stat.lookback_window
            );
        }
        Err(_) => {}
    }
    Ok(())
}
