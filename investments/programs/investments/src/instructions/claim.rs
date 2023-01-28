use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{instruction::Instruction, system_program},
    },
    anchor_spl::token::{transfer, TokenAccount, Transfer},
    clockwork_sdk::state::{Thread, ThreadAccount, ThreadResponse},
};

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(
        mut,
        associated_token::mint = investment.coin_mint,
        associated_token::authority = investment.authority
    )]
    pub authority_coin_vault: Account<'info, TokenAccount>,

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
    pub market: AccountInfo<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, Claim<'info>>) -> Result<ThreadResponse> {
    // Get accounts
    let authority_coin_vault = &mut ctx.accounts.authority_coin_vault;
    let investment = &ctx.accounts.investment;
    let investment_coin_vault = &mut ctx.accounts.investment_coin_vault;
    let investment_thread = &ctx.accounts.investment_thread;
    let market = &ctx.accounts.market;
    let token_program = &ctx.accounts.token_program;

    // get investment bump
    let bump = *ctx.bumps.get("investment").unwrap();

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

    let mut settle_funds_account_metas = vec![
        AccountMeta::new_readonly(anchor_spl::dex::ID, false),
        AccountMeta::new_readonly(investment.key(), false),
        AccountMeta::new_readonly(investment_thread.key(), true),
        AccountMeta::new(market.key(), false),
        AccountMeta::new_readonly(system_program::ID, false),
        AccountMeta::new_readonly(anchor_spl::token::ID, false),
    ];

    let mut remaining_account_metas = 
        ctx
        .remaining_accounts
        .iter()
        .map(|acc| AccountMeta::new(acc.key(), false))
        .collect::<Vec<AccountMeta>>();

    settle_funds_account_metas.append(&mut remaining_account_metas);

    Ok(ThreadResponse {
        kickoff_instruction: Some(
            Instruction {
                    program_id: crate::ID,
                    accounts: settle_funds_account_metas,
                    data: clockwork_sdk::utils::anchor_sighash("settle_funds").into(),
                }
                .into(),
            ),
        next_instruction: None,
    })
}
