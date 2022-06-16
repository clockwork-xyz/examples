use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{instruction::Instruction, system_program, sysvar},
    },
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{self, Mint, TokenAccount, Transfer},
    },
    std::mem::size_of,
};

#[derive(Accounts)]
#[instruction(amount: u64, transfer_rate: u64)]
pub struct CreatePayment<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(
        seeds = [SEED_AUTHORITY], 
        bump
    )]
    pub authority: Account<'info, Authority>,

    #[account(address = sysvar::clock::ID)]
    pub clock: Sysvar<'info, Clock>,

    #[account(
        init,
        payer = sender,
        seeds = [SEED_ESCROW],
        bump,
        space = 8 + size_of::<Escrow>(),
    )]
    pub escrow: Box<Account<'info, Escrow>>,

    #[account(mut, has_one = authority)]
    pub manager: Account<'info, cronos_scheduler::state::Manager>,

    pub mint: Account<'info, Mint>,

    #[account()]
    pub recipient: AccountInfo<'info>,

    #[account(
        associated_token::authority = recipient,
        associated_token::mint = mint,
    )]
    pub recipient_token_account: Box<Account<'info, TokenAccount>>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = cronos_scheduler::ID)]
    pub scheduler_program: Program<'info, cronos_scheduler::program::CronosScheduler>,

    #[account(mut)]
    pub sender: Signer<'info>,

    #[account(
        mut,
        associated_token::authority = sender,
        associated_token::mint = mint,
    )]
    pub sender_token_account: Box<Account<'info, TokenAccount>>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,

    #[account(
        init,
        payer = sender,
        associated_token::authority = escrow,
        associated_token::mint = mint,
    )]
    pub vault: Box<Account<'info, TokenAccount>>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, CreatePayment<'info>>,
    amount: u64,
    transfer_rate: u64,
) -> Result<()> {
    // Get accounts
    let _associated_token_program = &ctx.accounts.associated_token_program;
    let authority = &ctx.accounts.authority;
    let clock = &ctx.accounts.clock;
    let escrow = &mut ctx.accounts.escrow;
    let manager = &mut ctx.accounts.manager;
    let mint = &ctx.accounts.mint;
    let recipient = &ctx.accounts.recipient;
    let recipient_token_account = &ctx.accounts.recipient_token_account;
    let _rent = &ctx.accounts.rent;
    let scheduler_program = &ctx.accounts.scheduler_program;
    let sender = &mut ctx.accounts.sender;
    let sender_token_account = &ctx.accounts.sender_token_account;
    let system_program = &ctx.accounts.system_program;
    let token_program = &ctx.accounts.token_program;
    let vault = &ctx.accounts.vault;    

    // Get remaining Accounts
    let create_payment_fee = ctx.remaining_accounts.get(0).unwrap();
    let create_payment_queue = ctx.remaining_accounts.get(1).unwrap();
    let create_payment_task = ctx.remaining_accounts.get(2).unwrap();

    // initialize Accounts
    escrow.new(
        amount,
        mint.key(),
        recipient.key(),
        sender.key(),
        transfer_rate,
    )?;

    // transfer funds from sender to vault token account
    token::transfer(
        CpiContext::new(
            token_program.to_account_info(), 
            Transfer {
                from: sender_token_account.to_account_info().clone(),
                to: vault.to_account_info().clone(),
                authority: sender.to_account_info().clone(),
            }
        ), 
        amount
    )?;

    // get authority bump
    let bump = *ctx.bumps.get("authority").unwrap();

    // Create queue
    cronos_scheduler::cpi::queue_new(
        CpiContext::new_with_signer(
            scheduler_program.to_account_info(),
            cronos_scheduler::cpi::accounts::QueueNew {
                authority: authority.to_account_info(),
                clock: clock.to_account_info(),
                fee: create_payment_fee.to_account_info(),
                manager: manager.to_account_info(),
                payer: sender.to_account_info(),
                queue: create_payment_queue.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[&[SEED_AUTHORITY, &[bump]]],
        ),
        "*/30 * * * * * *".into(),
    )?;

    // create ix
    let disburse_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority.key(), false),
            AccountMeta::new_readonly(escrow.key(), false),
            AccountMeta::new_readonly(manager.key(), true),
            AccountMeta::new_readonly(mint.key(), false),
            AccountMeta::new_readonly(recipient.key(), false),
            AccountMeta::new(recipient_token_account.key(), false),
            AccountMeta::new_readonly(sender.key(), false),
            AccountMeta::new(vault.key(), false),
            AccountMeta::new_readonly(token_program.key(), false),
        ],
        data: cronos_scheduler::anchor::sighash("disburse_payment").into(),
    };

    // Create task with the hello world ix and add it to the queue
    cronos_scheduler::cpi::task_new(
        CpiContext::new_with_signer(
            scheduler_program.to_account_info(),
            cronos_scheduler::cpi::accounts::TaskNew {
                authority: authority.to_account_info(),
                manager: manager.to_account_info(),
                payer: sender.to_account_info(),
                queue: create_payment_queue.to_account_info(),
                system_program: system_program.to_account_info(),
                task: create_payment_task.to_account_info(),
            },
            &[&[SEED_AUTHORITY, &[bump]]],
        ),
        vec![disburse_ix.into()],
    )?;

    Ok(())
}
