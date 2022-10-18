use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{system_program, sysvar},
    },
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{self, Mint, TokenAccount, spl_token::instruction::AuthorityType, SetAuthority},
    },
    std::mem::size_of,
};

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct CreatePayment<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = sender,
        seeds = [
            SEED_PAYMENT, 
            sender.key().as_ref(), 
            recipient.key().as_ref(), 
            mint.key().as_ref()
        ],
        bump,
        space = 8 + size_of::<Payment>(),
    )]
    pub payment: Account<'info, Payment>,

    /// CHECK: the recipient is validated by the seeds of the payment account
    #[account()]
    pub recipient: AccountInfo<'info>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(mut)]
    pub sender: Signer<'info>,

    #[account(
        mut, 
        token::authority = sender,
        token::mint = mint,
    )]
    pub sender_token_account: Account<'info, TokenAccount>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, CreatePayment<'info>>,
    amount: u64,
) -> Result<()> {
    // Get accounts
    let mint = &ctx.accounts.mint;
    let payment = &mut ctx.accounts.payment;
    let recipient = &ctx.accounts.recipient;
    let sender = &ctx.accounts.sender;
    let sender_token_account = &mut ctx.accounts.sender_token_account;
    let token_program = &ctx.accounts.token_program;

    // get payment bump
    let bump = *ctx.bumps.get("payment").unwrap();

    // initialize payment account
    payment.new(
        sender.key(),
        recipient.key(),
        mint.key(),
        amount,
    )?;

    // set authority to sender's token account
    token::set_authority(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            SetAuthority { 
                current_authority: sender.to_account_info(), 
                account_or_mint: sender_token_account.to_account_info() 
            }, 
            &[&[
                SEED_PAYMENT,
                payment.sender.as_ref(),
                payment.recipient.as_ref(),
                payment.mint.as_ref(),
                &[bump],
            ]]),
            AuthorityType::AccountOwner,
             Some(payment.key())
        )?;

    Ok(())
}




//  let disburse_payment_ix = Instruction {
//         program_id: crate::ID,
//         accounts: vec![
//             AccountMeta::new_readonly(associated_token::ID, false),
//             AccountMeta::new(escrow.key(), false),
//             AccountMeta::new_readonly(payment.mint, false),
//             AccountMeta::new(payment.key(), false),
//             AccountMeta::new_readonly(payment_queue.key(), true),
//             AccountMeta::new_readonly(payment.recipient, false),
//             AccountMeta::new(recipient_token_account.key(), false),
//             AccountMeta::new_readonly(payment.sender, false),
//             AccountMeta::new_readonly(token_program.key(), false),
//         ],
//         data: clockwork_sdk::queue_program::utils::anchor_sighash("disburse_payment").into(),
//     };

//     // Create queue
//     clockwork_sdk::queue_program::cpi::queue_create(
//         CpiContext::new_with_signer(
//             clockwork_program.to_account_info(),
//             clockwork_sdk::queue_program::cpi::accounts::QueueCreate {
//                 authority: payment.to_account_info(),
//                 payer: sender.to_account_info(),
//                 queue: payment_queue.to_account_info(),
//                 system_program: system_program.to_account_info(),
//             },
//             &[&[
//                 SEED_PAYMENT,
//                 payment.sender.as_ref(),
//                 payment.recipient.as_ref(),
//                 payment.mint.as_ref(),
//                 &[bump],
//             ]],
//         ),
//         "payment".into(),
//         disburse_payment_ix.into(),
//         Trigger::Cron {
//             schedule: payment.schedule.to_string(),
//             skippable: true,
//         },
//     )?;
