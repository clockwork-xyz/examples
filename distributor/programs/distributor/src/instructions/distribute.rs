use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{system_program, sysvar},
    },
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{self, Mint, MintTo, TokenAccount},
    },
    clockwork_sdk::{
        state::{Thread, ThreadAccount, ThreadResponse},
    },
};

#[derive(Accounts)]
pub struct Distribute<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(
        seeds = [SEED_DISTRIBUTOR, distributor.mint.as_ref(), distributor.authority.as_ref()],
        bump,
        has_one = mint,
        has_one = recipient,
    )]
    pub distributor: Account<'info, Distributor>,

    #[account(
        mut,
        signer,
        address = distributor_thread.pubkey(),
        constraint = distributor_thread.authority.eq(&distributor.authority),
     )]
    pub distributor_thread: Box<Account<'info, Thread>>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: manually validated against distributor account and recipient's token account
    pub recipient: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = recipient
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, token::Token>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, Distribute<'info>>) -> Result<ThreadResponse> {
    // get accounts
    let distributor = &ctx.accounts.distributor;
    let mint = &ctx.accounts.mint;
    let recipient_token_account = &mut ctx.accounts.recipient_token_account;
    let token_program = &ctx.accounts.token_program;

    // get distributor bump
    let bump = *ctx.bumps.get("distributor").unwrap();

    // mint to recipient' token account
    token::mint_to(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            MintTo {
                authority: distributor.to_account_info(),
                mint: mint.to_account_info(),
                to: recipient_token_account.to_account_info(),
            },
            &[&[
                SEED_DISTRIBUTOR,
                distributor.mint.as_ref(),
                distributor.authority.as_ref(),
                &[bump],
            ]],
        ),
        distributor.mint_amount,
    )?;

    Ok(ThreadResponse::default())
}
