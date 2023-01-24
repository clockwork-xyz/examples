use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{instruction::Instruction, system_program, sysvar},
    },
    anchor_spl::token::{transfer, TokenAccount, Transfer},
    clockwork_sdk::state::{Thread, ThreadAccount, ThreadResponse},
};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        associated_token::mint = investment.mint_a,
        associated_token::authority = investment.authority
    )]
    pub authority_mint_a_vault: Account<'info, TokenAccount>,

    #[account(
        seeds = [SEED_INVESTMENT, investment.authority.key().as_ref(), investment.market.key().as_ref()],
        bump,
        has_one = market,
    )]
    pub investment: Account<'info, Investment>,

    #[account(
        associated_token::mint = investment.mint_a,
        associated_token::authority = investment
    )]
    pub investment_mint_a_vault: Account<'info, TokenAccount>,

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

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, Deposit<'info>>) -> Result<ThreadResponse> {
    // Get accounts
    let investment = &ctx.accounts.investment;
    let investment_mint_a_vault = &mut ctx.accounts.investment_mint_a_vault;
    let investment_thread = &ctx.accounts.investment_thread;
    let authority_mint_a_vault = &ctx.accounts.authority_mint_a_vault;
    let token_program = &ctx.accounts.token_program;

    // get investment bump
    let bump = *ctx.bumps.get("investment").unwrap();

    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: authority_mint_a_vault.to_account_info(),
                to: investment_mint_a_vault.to_account_info(),
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

    Ok(ThreadResponse {
        kickoff_instruction: None,
        next_instruction: Some(
            Instruction {
                program_id: crate::ID,
                accounts: vec![
                    AccountMeta::new_readonly(anchor_spl::dex::ID, false),
                    AccountMeta::new_readonly(investment.key(), false),
                    AccountMeta::new_readonly(investment_thread.key(), true),
                    AccountMeta::new(investment_mint_a_vault.key(), false),
                    AccountMeta::new_readonly(sysvar::rent::ID, false),
                    AccountMeta::new_readonly(system_program::ID, false),
                    AccountMeta::new_readonly(token_program.key(), false),
                ],
                data: clockwork_sdk::utils::anchor_sighash("swap").into(),
            }
            .into(),
        ),
    })
}
