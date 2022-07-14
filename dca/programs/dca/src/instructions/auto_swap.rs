// use {
//     crate::state::*,
//     anchor_lang::{
//         prelude::*,
//         solana_program::{instruction::Instruction, system_program, sysvar},
//     },
//     anchor_spl::{token::TokenAccount, associated_token::{self, AssociatedToken}},
// };

// #[derive(Accounts)]
// pub struct AutoSwap<'info> {
//     #[account(address = anchor_spl::associated_token::ID)]
//     pub associated_token_program: Program<'info, AssociatedToken>,

//     #[account(
//         seeds = [SEED_AUTHORITY],
//         bump
//     )]
//     pub authority: Account<'info, Authority>,

//     #[account(address = sysvar::clock::ID)]
//     pub clock: Sysvar<'info, Clock>,

//     #[account(
//         seeds = [SEED_ESCROW, sender.key().as_ref(), recipient.key().as_ref()],
//         bump
//     )]
//     pub escrow: Account<'info, Escrow>,

//     #[account(has_one = authority)]
//     pub manager: Account<'info, cronos_scheduler::state::Manager>,

//     #[account()]
//     pub recipient: AccountInfo<'info>,

//     #[account(
//         associated_token::authority = escrow.recipient,
//         associated_token::mint = escrow.mint,
//     )]
//     pub recipient_token_account: Box<Account<'info, TokenAccount>>,

//     #[account(address = cronos_scheduler::ID)]
//     pub scheduler_program: Program<'info, cronos_scheduler::program::CronosScheduler>,

//      #[account(mut)]
//     pub sender: Signer<'info>,

//     #[account(address = system_program::ID)]
//     pub system_program: Program<'info, System>,

//     #[account(address = anchor_spl::token::ID)]
//     pub token_program: Program<'info, anchor_spl::token::Token>,

//     #[account(
//         associated_token::authority = escrow,
//         associated_token::mint = escrow.mint,
//     )]
//     pub vault: Box<Account<'info, TokenAccount>>,
// }

// pub fn handler<'info> (
//   ctx: Context<'_, '_, '_, 'info, AutoSwap<'info>>
// ) -> Result<()> {
//     // Get Accounts
//     let authority = &ctx.accounts.authority;
//     let clock = &ctx.accounts.clock;
//     let escrow = &ctx.accounts.escrow;
//     let manager = &ctx.accounts.manager;
//     let recipient_token_account = &ctx.accounts.recipient_token_account;
//     let scheduler_program = &ctx.accounts.scheduler_program;
//     let sender = &ctx.accounts.sender;
//     let system_program = &ctx.accounts.system_program;
//     let token_program = &ctx.accounts.token_program;
//     let vault = &ctx.accounts.vault;

//     // Get remaining Accounts
//     let disburse_fee = ctx.remaining_accounts.get(0).unwrap();
//     let disburse_queue = ctx.remaining_accounts.get(1).unwrap();
//     let disburse_task = ctx.remaining_accounts.get(2).unwrap();

//     // get authority bump
//     let bump = *ctx.bumps.get("authority").unwrap();

//     // Create queue
//     cronos_scheduler::cpi::queue_new(
//         CpiContext::new_with_signer(
//             scheduler_program.to_account_info(),
//             cronos_scheduler::cpi::accounts::QueueNew {
//                 authority: authority.to_account_info(),
//                 clock: clock.to_account_info(),
//                 fee: disburse_fee.to_account_info(),
//                 manager: manager.to_account_info(),
//                 payer: sender.to_account_info(),
//                 queue: disburse_queue.to_account_info(),
//                 system_program: system_program.to_account_info(),
//             },
//             &[&[SEED_AUTHORITY, &[bump]]],
//         ),
//         "*/30 * * * * * *".into(),
//     )?;

//     // create swap ix
//     let swap_ix = Instruction {
//         program_id: crate::ID,
//         accounts: vec![
//             // AccountMeta::new_readonly(associated_token::ID, false),
//             // AccountMeta::new_readonly(authority.key(), false),
//             // AccountMeta::new(escrow.key(), false),
//             // AccountMeta::new_readonly(manager.key(), true),
//             // AccountMeta::new_readonly(escrow.mint.key(), false),
//             // AccountMeta::new_readonly(escrow.recipient.key(), false),
//             // AccountMeta::new(recipient_token_account.key(), false),
//             // AccountMeta::new_readonly(escrow.sender.key(), false),
//             // AccountMeta::new(vault.key(), false),
//             // AccountMeta::new_readonly(token_program.key(), false),
//         ],
//         data: cronos_scheduler::anchor::sighash("swap").into(),
//     };

//     // Create task with the disburse ix and add it to the queue
//     cronos_scheduler::cpi::task_new(
//         CpiContext::new_with_signer(
//             scheduler_program.to_account_info(),
//             cronos_scheduler::cpi::accounts::TaskNew {
//                 authority: authority.to_account_info(),
//                 manager: manager.to_account_info(),
//                 payer: sender.to_account_info(),
//                 queue: disburse_queue.to_account_info(),
//                 system_program: system_program.to_account_info(),
//                 task: disburse_task.to_account_info(),
//             },
//             &[&[SEED_AUTHORITY, &[bump]]],
//         ),
//         vec![swap_ix.into()],
//     )?;

//   Ok(())
// }
