use {
    crate::{error::ErrorCode, state::*},
    anchor_lang::prelude::*,
    anchor_spl::token::{transfer, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        address = Subscriber::pubkey(payer.key(),subscription.key()),
    )]
    pub subscriber: Account<'info, Subscriber>,
    #[account(
        mut,
        associated_token::authority = payer,
        associated_token::mint = subscription.mint,
    )]
    pub subscriber_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        address=Subscription::bank_pubkey(subscription.key(),subscription.owner.key())
    )]
    pub subscription_bank: Account<'info, TokenAccount>,

    #[account(mut, address = Subscription::pubkey(subscription.owner.key(),subscription.subscription_id.clone()))]
    pub subscription: Account<'info, Subscription>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> Withdraw<'_> {
    pub fn process(&mut self, amount: u64) -> Result<()> {
        let Self {
            subscriber,
            subscriber_token_account,
            subscription_bank,
            token_program,
            subscription,
            ..
        } = self;

        require!(
            subscriber.locked_amount >= amount,
            ErrorCode::InsuffiscientAmountLocked
        );

        subscriber.locked_amount -= amount;

        transfer(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                Transfer {
                    from: subscription_bank.to_account_info(),
                    to: subscriber_token_account.to_account_info(),
                    authority: subscription.to_account_info(),
                },
                &[&[
                    subscription.key().as_ref(),
                    subscription.owner.as_ref(),
                    "subscription_bank".as_bytes(),
                ]],
            ),
            amount,
        )?;

        Ok(())
    }
}
