use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{system_program, sysvar},
    },
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{self, spl_token::instruction::AuthorityType, Mint, SetAuthority, TokenAccount},
    },
    std::mem::size_of,
};

#[derive(Accounts)]
#[instruction(mint_amount: u64)]
pub struct Create<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        seeds = [SEED_DISTRIBUTOR, mint.key().as_ref(), authority.key().as_ref()],
        bump,
        payer = authority,
        space = 8 + size_of::<Distributor>(),
    )]
    pub distributor: Account<'info, Distributor>,

    /// CHECK: manually validated against recipient's token account
    #[account()]
    pub recipient: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = recipient
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, Create<'info>>,
    mint_amount: u64,
) -> Result<()> {
    // get accounts
    let authority = &ctx.accounts.authority;
    let distributor = &mut ctx.accounts.distributor;
    let mint = &mut ctx.accounts.mint;
    let recipient = &ctx.accounts.recipient;
    let recipient_token_account = &ctx.accounts.recipient_token_account;
    let token_program = &ctx.accounts.token_program;

    // initialize distributor account
    distributor.new(
        authority.key(),
        recipient.key(),
        recipient_token_account.key(),
        mint.key(),
        mint_amount,
    )?;
    msg!("distributor: {:#?}", distributor);

    // delegate mint authority from payer (authority) to distributor account
    token::set_authority(
        CpiContext::new(
            token_program.to_account_info(),
            SetAuthority {
                account_or_mint: mint.to_account_info(),
                current_authority: authority.to_account_info(),
            },
        ),
        AuthorityType::MintTokens,
        Some(distributor.key()),
    )?;

    Ok(())
}
