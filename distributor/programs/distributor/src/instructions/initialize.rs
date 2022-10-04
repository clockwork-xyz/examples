use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{instruction::Instruction, system_program, sysvar},
    },
    anchor_spl::{
        associated_token::{self, AssociatedToken},
        token::{self, spl_token::instruction::AuthorityType, Mint, SetAuthority, TokenAccount},
    },
    clockwork_sdk::queue_program::{
        self,
        accounts::{Queue, Trigger},
        utils::PAYER_PUBKEY,
        QueueProgram,
    },
    std::mem::size_of,
};

#[derive(Accounts)]
#[instruction(mint_amount: u64)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(address = queue_program::ID)]
    pub clockwork_program: Program<'info, QueueProgram>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        address = Distributor::pubkey(mint.key(), authority.key()),
        payer = authority,
        space = 8 + size_of::<Distributor>(),
    )]
    pub distributor: Account<'info, Distributor>,

    #[account(address = Queue::pubkey(authority.key(), "distributor".into()))]
    pub distributor_queue: SystemAccount<'info>,

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
    ctx: Context<'_, '_, '_, 'info, Initialize<'info>>,
    mint_amount: u64,
) -> Result<()> {
    // get accounts
    let authority = &ctx.accounts.authority;
    let clockwork_program = &ctx.accounts.clockwork_program;
    let distributor = &mut ctx.accounts.distributor;
    let distributor_queue = &mut ctx.accounts.distributor_queue;
    let mint = &mut ctx.accounts.mint;
    let recipient = &ctx.accounts.recipient;
    let recipient_token_account = &ctx.accounts.recipient_token_account;
    let system_program = &ctx.accounts.system_program;
    let token_program = &ctx.accounts.token_program;

    // initialize distributor account
    distributor.new(authority.key(), recipient.key(), mint.key(), mint_amount)?;

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

    // get distributor bump
    let bump = *ctx.bumps.get("distributor").unwrap();

    // defin mint token ix
    let mint_token_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new_readonly(distributor.key(), false),
            AccountMeta::new(distributor_queue.key(), true),
            AccountMeta::new(mint.key(), false),
            AccountMeta::new(PAYER_PUBKEY, true),
            AccountMeta::new_readonly(recipient.key(), false),
            AccountMeta::new(recipient_token_account.key(), false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
        ],
        data: clockwork_sdk::queue_program::utils::anchor_sighash("mint_token").to_vec(),
    };

    // initialize distributor queue
    clockwork_sdk::queue_program::cpi::queue_create(
        CpiContext::new_with_signer(
            clockwork_program.to_account_info(),
            clockwork_sdk::queue_program::cpi::accounts::QueueCreate {
                authority: distributor.to_account_info(),
                payer: authority.to_account_info(),
                queue: distributor_queue.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[&[
                SEED_DISTRIBUTOR,
                distributor.mint.as_ref(),
                distributor.authority.as_ref(),
                &[bump],
            ]],
        ),
        "distributor".into(),
        mint_token_ix.into(),
        Trigger::Cron {
            schedule: "*/5 * * * * * *".into(),
            skippable: true,
        },
    )?;

    Ok(())
}
