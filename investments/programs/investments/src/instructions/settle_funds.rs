use {
    crate::state::{Investment, SEED_INVESTMENT},
    anchor_lang::{
        prelude::*,
        solana_program::{instruction::Instruction, system_program, sysvar},
    },
    anchor_spl::{
        associated_token::{self, AssociatedToken},
        dex::SettleFunds as SerumDexSettleFunds,
        token::{self, TokenAccount, Token},
    },
    clockwork_crank::state::{CrankResponse, Queue, SEED_QUEUE},
};

#[derive(Accounts)]
pub struct SettleFunds<'info> {    
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
    )]
    pub investment: Box<Account<'info, Investment>>,

    #[account(
        associated_token::authority = investment, 
        associated_token::mint = investment.mint_a
    )]
    pub investment_mint_a_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        associated_token::authority = investment, 
        associated_token::mint = investment.mint_b
    )]
    pub investment_mint_b_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
      seeds = [SEED_QUEUE, investment.key().as_ref(), "investment".as_ref()], 
      seeds::program = clockwork_crank::ID, 
      bump,
    )]
    pub investment_queue: Box<Account<'info, Queue>>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, Token>,}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, SettleFunds<'info>>) -> Result<CrankResponse> {
    // get accounts
    let dex_program = &ctx.accounts.dex_program;
    let investment = &ctx.accounts.investment;
    let investment_mint_a_token_account = &ctx.accounts.investment_mint_a_token_account;
    let investment_mint_b_token_account = &ctx.accounts.investment_mint_b_token_account;
    let investment_queue = &ctx.accounts.investment_queue;
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

    // settle funds cpi
    anchor_spl::dex::settle_funds(CpiContext::new_with_signer(
        dex_program.to_account_info(),
        SerumDexSettleFunds {
            market: market.to_account_info(),
            open_orders: open_orders.to_account_info(),
            open_orders_authority: investment.to_account_info(),
            coin_vault: mint_b_vault.to_account_info(),
            pc_vault: mint_a_vault.to_account_info(),
            coin_wallet: mint_b_wallet.to_account_info(),
            pc_wallet: mint_a_wallet.to_account_info(),
            vault_signer: vault_signer.to_account_info(),
            token_program: token_program.to_account_info(),
        },
        &[&[
            SEED_INVESTMENT,
            investment.payer.as_ref(),
            investment.mint_a.as_ref(),
            investment.mint_b.as_ref(),
            &[bump],
        ]],
    ))?;

    // return swap ix
    Ok(CrankResponse {
        next_instruction: Some(
            Instruction {
                program_id: crate::ID,
                accounts: vec![ 
                    AccountMeta::new_readonly(associated_token::ID, false),
                    AccountMeta::new_readonly(dex_program.key(), false),
                    AccountMeta::new_readonly(investment.key(), false),
                    AccountMeta::new(investment_mint_a_token_account.key(), false),
                    AccountMeta::new_readonly(investment_mint_b_token_account.key(), false),
                    AccountMeta::new(investment_queue.key(), true),
                    AccountMeta::new(clockwork_crank::payer::ID, true),
                    AccountMeta::new_readonly(sysvar::rent::ID, false),
                    AccountMeta::new_readonly(system_program::ID, false),
                    AccountMeta::new_readonly(token::ID, false),
                    // Extra Accounts
                    AccountMeta::new(market.key(), false),
                    AccountMeta::new(mint_a_vault.key(), false),
                    AccountMeta::new(mint_b_vault.key(), false),
                    AccountMeta::new(request_queue.key(), false),
                    AccountMeta::new(event_queue.key(), false),
                    AccountMeta::new(market_bids.key(), false),
                    AccountMeta::new(market_asks.key(), false),
                    AccountMeta::new(open_orders.key(), false),
                    AccountMeta::new(vault_signer.key(), false),
                    AccountMeta::new(mint_a_wallet.key(), false),
                    AccountMeta::new(mint_b_wallet.key(), false)
                ],
                data: clockwork_crank::anchor::sighash("swap").into(),
            }
            .into(),
        ),
    })
}
