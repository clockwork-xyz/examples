use {
    crate::{errors::*, state::*},
    anchor_lang::{
        prelude::*,
        solana_program::{instruction::Instruction, system_program, sysvar},
    },
    anchor_spl::token::{self, Mint, SetAuthority, TokenAccount, Transfer},
    spl_token::instruction::AuthorityType,
    std::mem::size_of,
};

#[derive(Accounts)]
#[instruction(deposit_amount: u64, transfer_rate: u64)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        seeds = [SEED_AUTHORITY],
        bump,
        payer = payer,
        space = 8 + size_of::<Authority>(),
    )]
    pub authority: Account<'info, Authority>,

    pub mint: Account<'info, Mint>,

    #[account(
        init,
        seeds = [b"token".as_ref()],
        bump,
        payer = payer,
        token::mint = mint,
        token::authority = payer,
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = payer_deposit_account.amount >= deposit_amount,
        constraint = payer_deposit_account.amount > transfer_rate @ EscrowError::InvalidTransferRate
    )]
    pub payer_deposit_account: Account<'info, TokenAccount>,

    // might not need this bih
    // pub initializer_receive_account: Account<'info, TokenAccount>,
    #[account(zero)]
    pub escrow: Box<Account<'info, Escrow>>,

    pub rent: Sysvar<'info, Rent>,

    pub token_program: AccountInfo<'info>,

    #[account(address = sysvar::clock::ID)]
    pub clock: Sysvar<'info, Clock>,

    #[account(address = cronos_scheduler::ID)]
    pub scheduler_program: Program<'info, cronos_scheduler::program::CronosScheduler>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    // sets ctx for authority to the vault account
    fn set_authority_ctx(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.vault.to_account_info().clone(),
            current_authority: self.payer.to_account_info().clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    // transfers from payer to vault account
    fn deposit_funds_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.payer_deposit_account.to_account_info().clone(),
            to: self.vault.to_account_info().clone(),
            authority: self.payer.to_account_info().clone(),
        };

        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, Initialize<'info>>,
    deposit_amount: u64,
    transfer_rate: u64,
) -> Result<()> {
    // Get accounts
    let payer = &ctx.accounts.payer;
    let authority = &mut ctx.accounts.authority;
    let mint = &ctx.accounts.mint;
    let vault = &ctx.accounts.vault;
    let initializer_deposit_account = &ctx.accounts.payer_deposit_account;
    let escrow = &mut ctx.accounts.escrow;
    let rent = &ctx.accounts.rent;
    let clock = &ctx.accounts.clock;
    let token_program = &ctx.accounts.token_program;
    let scheduler_program = &ctx.accounts.scheduler_program;
    let system_program = &ctx.accounts.system_program;

    // derive vault authority
    let vault_authority = Escrow::pda().0;

    // Get remaining Accounts
    let transfer_fee = ctx.remaining_accounts.get(0).unwrap();
    let transfer_queue = ctx.remaining_accounts.get(1).unwrap();
    let manager = ctx.remaining_accounts.get(2).unwrap();
    let transfer_task = ctx.remaining_accounts.get(3).unwrap();

    // initialize Accounts
    authority.new(manager.key())?;
    escrow.new(
        payer.key(),
        initializer_deposit_account.key(),
        deposit_amount,
        transfer_rate,
    )?;

    // going from &mut -> & to avoid mutable borrow later
    let authority = &ctx.accounts.authority;

    // set authority of vault to escrow
    token::set_authority(
        ctx.accounts.set_authority_ctx(),
        AuthorityType::AccountOwner,
        Some(vault_authority),
    )?;

    // transfer funds from payer to vault account
    token::transfer(ctx.accounts.deposit_funds_ctx(), deposit_amount)?;

    // Create Manager
    let bump = *ctx.bumps.get("authority").unwrap();
    cronos_scheduler::cpi::manager_new(CpiContext::new_with_signer(
        scheduler_program.to_account_info(),
        cronos_scheduler::cpi::accounts::ManagerNew {
            authority: authority.to_account_info(),
            manager: manager.to_account_info(),
            payer: payer.to_account_info(),
            system_program: system_program.to_account_info(),
        },
        &[&[SEED_AUTHORITY, &[bump]]],
    ))?;

    // Create queue
    cronos_scheduler::cpi::queue_new(
        CpiContext::new_with_signer(
            scheduler_program.to_account_info(),
            cronos_scheduler::cpi::accounts::QueueNew {
                authority: authority.to_account_info(),
                clock: clock.to_account_info(),
                fee: transfer_fee.to_account_info(),
                manager: manager.to_account_info(),
                payer: payer.to_account_info(),
                queue: transfer_queue.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[&[SEED_AUTHORITY, &[bump]]],
        ),
        "0 * * * * * *".into(),
    )?;

    // create ix
    let message_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![
            //TODO: THESE ACCOUNTS NEED TO BE UPDATED
            AccountMeta::new_readonly(authority.key(), false),
            AccountMeta::new_readonly(manager.key(), true),
            AccountMeta::new_readonly(sysvar::clock::ID, false),
        ],
        data: sighash("global", "transfer").into(),
    };

    // Create task with the hello world ix and add it to the queue
    cronos_scheduler::cpi::task_new(
        CpiContext::new_with_signer(
            scheduler_program.to_account_info(),
            cronos_scheduler::cpi::accounts::TaskNew {
                authority: authority.to_account_info(),
                manager: manager.to_account_info(),
                payer: payer.to_account_info(),
                queue: transfer_queue.to_account_info(),
                system_program: system_program.to_account_info(),
                task: transfer_task.to_account_info(),
            },
            &[&[SEED_AUTHORITY, &[bump]]],
        ),
        vec![message_ix.into()],
    )?;

    Ok(())
}

fn sighash(namespace: &str, name: &str) -> [u8; 8] {
    let preimage = format!("{}:{}", namespace, name);
    let mut sighash = [0u8; 8];
    sighash.copy_from_slice(
        &anchor_lang::solana_program::hash::hash(preimage.as_bytes()).to_bytes()[..8],
    );
    sighash
}
