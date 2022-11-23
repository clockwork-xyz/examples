use {crate::state::*, anchor_lang::prelude::*};

#[derive(Accounts)]
#[instruction(amount: Option<u64>)]
pub struct UpdatePayment<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SEED_PAYMENT,
            payment.authority.key().as_ref(),
            payment.mint.key().as_ref(),
            payment.recipient.key().as_ref(),
        ],
        bump,
        has_one = authority,
    )]
    pub payment: Account<'info, Payment>,
}

pub fn handler<'info>(ctx: Context<UpdatePayment>, amount: Option<u64>) -> Result<()> {
    // Get accounts
    let payment = &mut ctx.accounts.payment;

    // Update the payment amount.
    if let Some(amount) = amount {
        payment.amount = amount;
    }

    Ok(())
}
