// use {
//     crate::state::*,
//     anchor_lang::{
//         prelude::*,
//         solana_program::system_program,
//     },
//     anchor_spl::token::Mint,
//     clockwork_scheduler::{state::{SEED_QUEUE, Queue}, program::ClockworkScheduler},
// };

// #[derive(Accounts)]
// #[instruction(disbursement_amount: Option<u64>, schedule: Option<String>)]
// pub struct UpdatePayment<'info> {
//     pub mint: Account<'info, Mint>,

//     #[account(
//         mut,
//         seeds = [SEED_PAYMENT, payment.sender.key().as_ref(), payment.recipient.key().as_ref(), payment.mint.as_ref()],
//         bump,
//         has_one = recipient,
//         has_one = sender,
//         has_one = mint
//     )]
//     pub payment: Account<'info, Payment>,

//     #[account(
//         mut,
//         seeds = [SEED_QUEUE, payment.key().as_ref(), "payment_queue".as_bytes()],
//         seeds::program = clockwork_scheduler::ID,
//         bump,
// 	  )]
//     pub queue: Account<'info, Queue>,

//     #[account()]
//     pub recipient: AccountInfo<'info>,

//     #[account(address = clockwork_scheduler::ID)]
//     pub scheduler_program: Program<'info, ClockworkScheduler>,

//     #[account(mut)]
//     pub sender: Signer<'info>,

//     #[account(address = system_program::ID)]
//     pub system_program: Program<'info, System>,

// }

// pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, UpdatePayment<'info>>, disbursement_amount: Option<u64>, schedule: Option<String>) -> Result<()> {
//     // Get accounts
//     let payment = &mut ctx.accounts.payment;
//     let queue = &mut ctx.accounts.queue;
//     let scheduler_program = &ctx.accounts.scheduler_program;

//     // get payment bump
//     let bump = *ctx.bumps.get("payment").unwrap();

//     match disbursement_amount {
//         Some(da) => payment.disbursement_amount = da,
//         None => {}
//     }

//     match schedule {
//       Some(s) => {
//         // Update queue schedule
//         clockwork_scheduler::cpi::queue_update(
//             CpiContext::new_with_signer(
//                 scheduler_program.to_account_info(),
//                 clockwork_scheduler::cpi::accounts::QueueUpdate {
//                     authority: payment.to_account_info(),
//                     queue: queue.to_account_info(),
//                 },
//                 &[&[SEED_PAYMENT, payment.sender.as_ref(), payment.recipient.as_ref(), payment.mint.as_ref(), &[bump]]]
//             ),
//             s.to_string(),
//         )?;
//       },
//       None => {}
//     }

//     Ok(())
// }
