use {
    crate::state::{Authority, SEED_AUTHORITY},
    anchor_lang::{
        prelude::*,
        solana_program::{system_program, sysvar},
    },
    anchor_spl::{
        dex::{
            serum_dex::{
                instruction::SelfTradeBehavior,
                matching::{OrderType, Side},
            },
            NewOrderV3,
        },
        token::{Mint, Token, TokenAccount},
    },
    cronos_scheduler::state::Manager,
    std::num::NonZeroU64,
};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(seeds = [SEED_AUTHORITY, authority.payer.as_ref()], bump, has_one = manager)]
    pub authority: Account<'info, Authority>,

    #[account(address = anchor_spl::dex::ID)]
    pub dex_program: Program<'info, anchor_spl::dex::Dex>,

    #[account(signer, has_one = authority)]
    pub manager: Account<'info, Manager>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account()]
    pub pc_mint: Account<'info, Mint>,

    #[account(mut, token::authority = authority, token::mint = pc_mint)]
    pub pc_wallet: Account<'info, TokenAccount>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, Token>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, Swap<'info>>) -> Result<()> {
    // get accounts
    let authority = &ctx.accounts.authority;
    let dex_program = &ctx.accounts.dex_program;
    let pc_wallet = &mut ctx.accounts.pc_wallet;
    let rent = &ctx.accounts.rent;
    let token_program = &ctx.accounts.token_program;

    // get remaining accounts
    let market = ctx.remaining_accounts.get(0).unwrap();
    let coin_vault = ctx.remaining_accounts.get(1).unwrap();
    let pc_vault = ctx.remaining_accounts.get(2).unwrap();
    let request_queue = ctx.remaining_accounts.get(3).unwrap();
    let event_queue = ctx.remaining_accounts.get(4).unwrap();
    let market_bids = ctx.remaining_accounts.get(5).unwrap();
    let market_asks = ctx.remaining_accounts.get(6).unwrap();
    let open_orders = ctx.remaining_accounts.get(7).unwrap();

    // get authority bump
    let bump = *ctx.bumps.get("authority").unwrap();

    // make cpi to serum dex to swap
    anchor_spl::dex::new_order_v3(
        CpiContext::new_with_signer(
            dex_program.to_account_info(),
            NewOrderV3 {
                market: market.to_account_info(),
                coin_vault: coin_vault.to_account_info(),
                pc_vault: pc_vault.to_account_info(),
                request_queue: request_queue.to_account_info(),
                event_queue: event_queue.to_account_info(),
                market_bids: market_bids.to_account_info(),
                market_asks: market_asks.to_account_info(),
                open_orders: open_orders.to_account_info(),
                order_payer_token_account: pc_wallet.to_account_info(),
                open_orders_authority: authority.to_account_info(),
                token_program: token_program.to_account_info(),
                rent: rent.to_account_info(),
            },
            &[&[SEED_AUTHORITY, authority.payer.as_ref(), &[bump]]],
        ),
        Side::Bid,
        NonZeroU64::new(500).unwrap(),
        NonZeroU64::new(1_000).unwrap(),
        NonZeroU64::new(500_000).unwrap(),
        SelfTradeBehavior::DecrementTake,
        OrderType::Limit,
        019269,
        std::u16::MAX,
    )?;

    Ok(())
}
