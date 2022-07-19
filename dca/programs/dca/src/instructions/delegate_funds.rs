use {
    cronos_scheduler::state::Manager,
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{system_program, sysvar},
    },
    anchor_spl::token::{
        self, spl_token::instruction::AuthorityType, Mint, SetAuthority, TokenAccount,
    },
};

#[derive(Accounts)]

pub struct DelegateFunds<'info> {
    #[account( 
        seeds = [SEED_AUTHORITY],
        bump
    )]
    pub authority: Account<'info, Authority>,

    #[account(has_one = authority)]
    pub manager: Account<'info, Manager>,

    #[account(
        token::authority = payer,
        token::mint = pc_mint
    )]
    pub pc_wallet: Account<'info, TokenAccount>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account()]
    pub pc_mint: Account<'info, Mint>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, DelegateFunds<'info>>) -> Result<()> {
    // Get accounts
    let manager = &mut ctx.accounts.manager;
    let payer = &mut ctx.accounts.payer;
    let pc_wallet = &ctx.accounts.pc_wallet;
    let token_program = &ctx.accounts.token_program;

    // set authority of payers token account to manager
    token::set_authority(
        CpiContext::new(
            token_program.to_account_info(),
            SetAuthority {
                account_or_mint: pc_wallet.to_account_info(),
                current_authority: payer.to_account_info(),
            },
        ),
        AuthorityType::AccountOwner,
        Some(manager.key()),
    )?;

    Ok(())
}
