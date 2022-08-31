use anchor_spl::associated_token;

use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{instruction::Instruction, system_program, sysvar},
    },
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{self, Mint, TokenAccount, SetAuthority, spl_token::instruction::AuthorityType},
    },
    clockwork_crank::{
        program::ClockworkCrank,
        state::{Trigger, SEED_QUEUE},
    },
    std::mem::size_of,
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>, 

    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(address = clockwork_crank::ID)]
    pub clockwork_program: Program<'info, ClockworkCrank>,

    #[account(
        init,
        seeds = [SEED_DISTRIBUTOR, mint.key().as_ref(), admin.key().as_ref()],
        bump,
        payer = admin,
        space = 8 + size_of::<Distributor>(),
    )]
    pub distributor: Account<'info, Distributor>,

    #[account(
        seeds = [
            SEED_QUEUE, 
            distributor.key().as_ref(), 
            "distributor".as_bytes()
        ], 
        seeds::program = clockwork_crank::ID,
        bump
     )]
    pub distributor_queue: SystemAccount<'info>,

    /// CHECK: manually validated against distributor account and recipient's token account
    #[account()]
    pub mint: Account<'info, Mint>,
     
    /// CHECK: manually validated against recipient's token account
    #[account()]
    pub recipient: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = admin,
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

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, Initialize<'info>>) -> Result<()> {
    // get accounts
    let admin = &ctx.accounts.admin;
    let clockwork_program = &ctx.accounts.clockwork_program;
    let distributor = &mut ctx.accounts.distributor;
    let distributor_queue = &mut ctx.accounts.distributor_queue;
    let mint = &ctx.accounts.mint;
    let recipient = &ctx.accounts.recipient;
    let recipient_token_account = &ctx.accounts.recipient_token_account;
    let system_program = &ctx.accounts.system_program;
    let token_program = &ctx.accounts.token_program;

    // initialize distributor account
    distributor.new(admin.key(), recipient.key(), mint.key())?;

    // delegate mint authority from payer (admin) to distributor account
    token::set_authority(
    CpiContext::new(
        token_program.to_account_info(),
        SetAuthority {
                    account_or_mint: mint.to_account_info(), 
                    current_authority: admin.to_account_info()
                }), 
        AuthorityType::AccountOwner,
        Some(distributor.key())
    )?;

    // get distributor bump
    let bump = *ctx.bumps.get("distributor").unwrap();

    // defin mint token ix
    let mint_token_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new_readonly(admin.key(), false),
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new_readonly(distributor.key(), false),
            AccountMeta::new(distributor_queue.key(), true),
            AccountMeta::new_readonly(mint.key(), false),
            AccountMeta::new(clockwork_crank::payer::ID, true),
            AccountMeta::new_readonly(recipient.key(), false),
            AccountMeta::new(recipient_token_account.key(), false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),

        ],
        data: clockwork_crank::anchor::sighash("mint_token").to_vec()
    };
    
    // initialize distributor queue
    clockwork_crank::cpi::queue_create(
        CpiContext::new_with_signer(
            clockwork_program.to_account_info(),
            clockwork_crank::cpi::accounts::QueueCreate {
                authority: distributor.to_account_info(),
                payer: admin.to_account_info(),
                queue: distributor_queue.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[&[SEED_DISTRIBUTOR, distributor.admin.as_ref(), distributor.mint.as_ref(), &[bump]]],
        ),
        mint_token_ix.into(),
        "distributor".into(),
        Trigger::Cron {
            schedule: "*/15 * * * * * *".into(),
        },
    )?;

    Ok(())
}
