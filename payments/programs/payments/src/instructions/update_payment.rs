use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::system_program},
    anchor_spl::token::Mint,
};

#[derive(Accounts)]
#[instruction(amount: Option<u64>)]
pub struct UpdatePayment<'info> {
    #[account()]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [
            SEED_PAYMENT,
            payment.sender.key().as_ref(),
            payment.recipient.key().as_ref(),
            payment.mint.key().as_ref()
        ],
        bump,
        has_one = recipient,
        has_one = sender,
        has_one = mint
    )]
    pub payment: Account<'info, Payment>,

    /// CHECK: this account is validated against the payment account
    #[account()]
    pub recipient: AccountInfo<'info>,

    #[account(mut)]
    pub sender: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, UpdatePayment<'info>>,
    amount: Option<u64>,
) -> Result<()> {
    // Get payment account
    let payment = &mut ctx.accounts.payment;

    // update amount
    if let Some(amount) = amount {
        payment.amount = amount;
    }

    Ok(())
}
