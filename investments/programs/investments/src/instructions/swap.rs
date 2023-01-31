use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        __private::bytemuck::Contiguous,
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
        token::{Token, TokenAccount, transfer, Transfer},
    },
    std::num::NonZeroU64,
    clockwork_sdk::state::{Thread, ThreadAccount},
};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(
        mut,
        associated_token::mint = investment.pc_mint,
        associated_token::authority = investment.authority
    )]
    pub authority_pc_vault: Box<Account<'info, TokenAccount>>,

    #[account(address = anchor_spl::dex::ID)]
    pub dex_program: Program<'info, OpenBookDex>,

    #[account(
        seeds = [
            SEED_INVESTMENT, 
            investment.authority.key().as_ref(), 
            investment.market.key().as_ref(), 
        ], 
        bump,
    )]
    pub investment: Box<Account<'info, Investment>>,
    
    #[account(
        mut,
        associated_token::authority = investment,
        associated_token::mint = investment.pc_mint,
    )]
    pub investment_pc_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        signer,
        address = investment_thread.pubkey(),
        constraint = investment_thread.authority == investment.authority
    )]
    pub investment_thread: Box<Account<'info, Thread>>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, Token>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, Swap<'info>>) -> Result<()> {
    // get accounts
    let authority_pc_vault = &mut ctx.accounts.authority_pc_vault; 
    let dex_program = &ctx.accounts.dex_program;
    let investment = &ctx.accounts.investment;
    let investment_pc_vault= &mut ctx.accounts.investment_pc_vault;
    let rent = &ctx.accounts.rent;
    let token_program = &ctx.accounts.token_program;

    // get remaining accounts
    let market = &mut ctx.remaining_accounts.get(0).unwrap();
    let event_queue = &mut ctx.remaining_accounts.get(1).unwrap();
    let request_queue = &mut ctx.remaining_accounts.get(2).unwrap();
    let market_bids = &mut ctx.remaining_accounts.get(3).unwrap();
    let market_asks = &mut ctx.remaining_accounts.get(4).unwrap();
    let coin_vault = &mut ctx.remaining_accounts.get(5).unwrap();
    let pc_vault = &mut ctx.remaining_accounts.get(6).unwrap();
    let open_orders = &mut ctx.remaining_accounts.get(7).unwrap();
    
    // get investment bump
    let bump = *ctx.bumps.get("investment").unwrap();

    // transfer swap amount from authority to investment ata
    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: authority_pc_vault.to_account_info(),
                to: investment_pc_vault.to_account_info(),
                authority: investment.to_account_info(),
            },
            &[&[
                SEED_INVESTMENT,
                investment.authority.as_ref(),
                investment.market.as_ref(),
                &[bump],
            ]],
        ),
        investment.swap_amount,
    )?;

    // place order on serum dex
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
                order_payer_token_account: investment_pc_vault.to_account_info(),
                open_orders_authority: investment.to_account_info(),
                token_program: token_program.to_account_info(),
                rent: rent.to_account_info(),
            },
            &[&[
                SEED_INVESTMENT,
                investment.authority.as_ref(),
                investment.market.as_ref(),
                &[bump],
            ]],
        ),
        Side::Bid,
        NonZeroU64::new(NonZeroU64::MAX_VALUE).unwrap(),
        NonZeroU64::new(NonZeroU64::MAX_VALUE).unwrap(),
        NonZeroU64::new(investment.swap_amount).unwrap(),
        SelfTradeBehavior::DecrementTake,
        OrderType::Limit,
        019269,
        std::u16::MAX,
    )?;

    Ok(())
}