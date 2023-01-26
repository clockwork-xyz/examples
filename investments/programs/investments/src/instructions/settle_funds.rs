use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::{system_program, instruction::Instruction}},
    anchor_spl::{dex::{settle_funds, SettleFunds as OpenbookDexSettleFunds}, associated_token},
    clockwork_sdk::state::{Thread, ThreadAccount, ThreadResponse}
};

#[derive(Accounts)]
pub struct SettleFunds<'info> {
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
        signer,
        address = investment_thread.pubkey(),
        constraint = investment_thread.authority == investment.authority
    )]
    pub investment_thread: Account<'info, Thread>,

    /// CHECK: manually checked against investment acc
    pub market: AccountInfo<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, SettleFunds<'info>>) -> Result<ThreadResponse> {
    // Get accounts
    let dex_program = &ctx.accounts.dex_program;
    let investment = &ctx.accounts.investment;
    let investment_thread = &ctx.accounts.investment_thread;
    let market = &ctx.accounts.market;
    let token_program = &ctx.accounts.token_program;

    // Get remaining accounts
    let open_orders = ctx.remaining_accounts.get(0).unwrap();
    let coin_vault = ctx.remaining_accounts.get(1).unwrap();
    let coin_wallet = ctx.remaining_accounts.get(2).unwrap();
    let pc_vault = ctx.remaining_accounts.get(3).unwrap();
    let pc_wallet = ctx.remaining_accounts.get(4).unwrap();
    let vault_signer = ctx.remaining_accounts.get(5).unwrap();

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

    let authority_mint_b_vault = 
        associated_token::get_associated_token_address(&investment.authority, &investment.mint_b);
    let investment_mint_b_vault = 
        associated_token::get_associated_token_address(&investment.key(), &investment.mint_b);

     Ok(ThreadResponse {
        kickoff_instruction: None,
        next_instruction: Some(
            Instruction {
                program_id: crate::ID,
                accounts: vec![
                    AccountMeta::new(authority_mint_b_vault, false),
                    AccountMeta::new_readonly(investment.key(), false),
                    AccountMeta::new(investment_mint_b_vault.key(), false),
                    AccountMeta::new_readonly(investment_thread.key(), true),
                    AccountMeta::new_readonly(investment.market, false),
                    AccountMeta::new_readonly(system_program::ID, false),
                    AccountMeta::new_readonly(token_program.key(), false),
                    // REMAINING ACCOUNTS
                    AccountMeta::new_readonly(open_orders.key(), false),
                    AccountMeta::new_readonly(coin_vault.key(), false),
                    AccountMeta::new_readonly(coin_wallet.key(), false),
                    AccountMeta::new_readonly(pc_vault.key(), false),
                    AccountMeta::new_readonly(pc_wallet.key(), false),
                    AccountMeta::new_readonly(vault_signer.key(), false),
                ],
                data: clockwork_sdk::utils::anchor_sighash("claim").into(),
            }
            .into(),
        ),
    })
}
