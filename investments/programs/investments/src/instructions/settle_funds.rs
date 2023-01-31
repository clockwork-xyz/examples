use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::system_program},
    anchor_spl::{
        dex::{settle_funds, SettleFunds as OpenbookDexSettleFunds}, 
        token::{transfer, TokenAccount, Transfer}
    },
    clockwork_sdk::state::{Thread, ThreadAccount}
};

#[derive(Accounts)]
pub struct SettleFunds<'info> {
    #[account(
        mut,
        associated_token::mint = investment.coin_mint,
        associated_token::authority = investment.authority
    )]
    pub authority_coin_vault: Account<'info, TokenAccount>,

    #[account(address = anchor_spl::dex::ID)]
    pub dex_program: Program<'info, OpenBookDex>,

    #[account(
        seeds = [
            SEED_INVESTMENT, 
            investment.authority.key().as_ref(), 
            investment.market.key().as_ref()
        ],
        bump,
        has_one = market,
    )]
    pub investment: Account<'info, Investment>,

    #[account(  
        mut,
        associated_token::mint = investment.coin_mint,
        associated_token::authority = investment
    )]
    pub investment_coin_vault: Account<'info, TokenAccount>,

    #[account(
        signer,
        address = investment_thread.pubkey(),
        constraint = investment_thread.authority == investment.authority
    )]
    pub investment_thread: Account<'info, Thread>,

    /// CHECK: manually checked against investment acc
    #[account(mut)]
    pub market: AccountInfo<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, SettleFunds<'info>>) -> Result<()> {
    // Get accounts
    let authority_coin_vault = &mut ctx.accounts.authority_coin_vault;
    let dex_program = &ctx.accounts.dex_program;
    let investment = &ctx.accounts.investment;
    let investment_coin_vault = &mut ctx.accounts.investment_coin_vault;
    let market = &mut ctx.accounts.market;
    let token_program = &ctx.accounts.token_program;

    // Get remaining accounts
    let open_orders = &mut ctx.remaining_accounts.get(0).unwrap();
    let coin_vault = &mut ctx.remaining_accounts.get(1).unwrap();
    let coin_wallet = &mut ctx.remaining_accounts.get(2).unwrap();
    let pc_vault = &mut ctx.remaining_accounts.get(3).unwrap();
    let pc_wallet = &mut ctx.remaining_accounts.get(4).unwrap();
    let vault_signer = &mut ctx.remaining_accounts.get(5).unwrap();

    // get investment bump
    let bump = *ctx.bumps.get("investment").unwrap();

    settle_funds(CpiContext::new_with_signer(
        dex_program.to_account_info(),
        OpenbookDexSettleFunds {
            market: market.to_account_info(),
            token_program: token_program.to_account_info(),
            open_orders: open_orders.to_account_info(),
            open_orders_authority: investment.to_account_info(),
            coin_vault: coin_vault.to_account_info(),
            coin_wallet: coin_wallet.to_account_info(),
            pc_vault: pc_vault.to_account_info(),
            pc_wallet: pc_wallet.to_account_info(),
            vault_signer: vault_signer.to_account_info(),
        },
        &[&[
            SEED_INVESTMENT,
            investment.authority.as_ref(),
            investment.market.as_ref(),
            &[bump],
        ]],
    ))?;


    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: investment_coin_vault.to_account_info(),
                to: authority_coin_vault.to_account_info(),
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


     Ok(())
}
