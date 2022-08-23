use anchor_spl::dex::SettleFunds;

use {
    crate::state::{Investment, SEED_INVESTMENT},
    anchor_lang::{
        prelude::*,
        solana_program::{system_program, sysvar},
    },
    anchor_spl::{ 
        associated_token::AssociatedToken,
        dex::{
            serum_dex::{
                instruction::SelfTradeBehavior,
                matching::{OrderType, Side},
            },
            NewOrderV3,
        },
        token::{Mint, Token, TokenAccount},
    },
    clockwork_scheduler::state::{Queue, SEED_QUEUE},
    std::num::NonZeroU64,
};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(address = anchor_spl::dex::ID)]
    pub dex_program: Program<'info, anchor_spl::dex::Dex>,

    #[account(
        seeds = [
            SEED_INVESTMENT, 
            investment.payer.as_ref(), 
            investment.mint_a.as_ref(), 
            investment.mint_b.as_ref()
        ],
        bump,
        has_one = mint_a
    )]
    pub investment: Account<'info, Investment>,

    #[account(
        mut, 
        associated_token::authority = investment, 
        associated_token::mint = investment.mint_a
    )]
    pub investment_mint_a_token_accoount: Account<'info, TokenAccount>,

    #[account(
        mut, 
        associated_token::authority = investment, 
        associated_token::mint = investment.mint_b
    )]
    pub investment_mint_b_token_accoount: Account<'info, TokenAccount>,

    #[account()]
    pub mint_a: Account<'info, Mint>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
      seeds = [SEED_QUEUE, investment.key().as_ref(), "investment_queue".as_ref()], 
      seeds::program = clockwork_scheduler::ID, 
      bump,
    )]
    pub queue: Account<'info, Queue>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, Token>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, Swap<'info>>) -> Result<()> {
    // get accounts
    let dex_program = &ctx.accounts.dex_program;
    let investment = &ctx.accounts.investment;
    let investment_mint_a_token_accoount = &mut ctx.accounts.investment_mint_a_token_accoount;
    // let investment_mint_b_token_accoount = &mut ctx.accounts.investment_mint_b_token_accoount;
    let rent = &ctx.accounts.rent;
    let token_program = &ctx.accounts.token_program;

    // get remaining accounts
    let market = ctx.remaining_accounts.get(0).unwrap();
    let mint_a_vault = ctx.remaining_accounts.get(1).unwrap();
    let mint_b_vault = ctx.remaining_accounts.get(2).unwrap();
    let request_queue = ctx.remaining_accounts.get(3).unwrap();
    let event_queue = ctx.remaining_accounts.get(4).unwrap();
    let market_bids = ctx.remaining_accounts.get(5).unwrap();
    let market_asks = ctx.remaining_accounts.get(6).unwrap();
    let open_orders = ctx.remaining_accounts.get(7).unwrap();
    let vault_signer = ctx.remaining_accounts.get(8).unwrap();
    let mint_a_wallet = ctx.remaining_accounts.get(9).unwrap();
    let mint_b_wallet = ctx.remaining_accounts.get(10).unwrap();

    // get investment bump
    let bump = *ctx.bumps.get("investment").unwrap();

    // place order on serum dex
    anchor_spl::dex::new_order_v3(
        CpiContext::new_with_signer(
            dex_program.to_account_info(),
            NewOrderV3 {
                market: market.to_account_info(),
                coin_vault: mint_b_vault.to_account_info(),
                pc_vault: mint_a_vault.to_account_info(),
                request_queue: request_queue.to_account_info(),
                event_queue: event_queue.to_account_info(),
                market_bids: market_bids.to_account_info(),
                market_asks: market_asks.to_account_info(),
                open_orders: open_orders.to_account_info(),
                order_payer_token_account: investment_mint_a_token_accoount.to_account_info(),
                open_orders_authority: investment.to_account_info(),
                token_program: token_program.to_account_info(),
                rent: rent.to_account_info(),
            },
            &[&[
                SEED_INVESTMENT,
                investment.payer.as_ref(),
                investment.mint_a.as_ref(),
                investment.mint_b.as_ref(),
                &[bump],
            ]],
        ),
        Side::Bid,
            NonZeroU64::new(u64::MAX).unwrap(),
            NonZeroU64::new(u64::MAX).unwrap(),
        NonZeroU64::new(investment.swap_amount).unwrap(),
        SelfTradeBehavior::DecrementTake,
        OrderType::Limit,
        019269,
        std::u16::MAX,
    )?;
    
    // settle funds
    anchor_spl::dex::settle_funds(
        CpiContext::new_with_signer(
            dex_program.to_account_info(), 
            SettleFunds { 
                market: market.to_account_info(), 
                open_orders: open_orders.to_account_info(), 
                open_orders_authority: investment.to_account_info(), 
                coin_vault: mint_b_vault.to_account_info(), 
                pc_vault: mint_a_vault.to_account_info(), 
                coin_wallet: mint_b_wallet.to_account_info(),
                pc_wallet: mint_a_wallet.to_account_info(), 
                vault_signer: vault_signer.to_account_info(), 
                token_program: token_program.to_account_info() 
            }, 
    &[&[
                SEED_INVESTMENT,
                investment.payer.as_ref(),
                investment.mint_a.as_ref(),
                investment.mint_b.as_ref(),
                &[bump],
            ]],
        ))?;

    Ok(())
}

