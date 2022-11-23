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
        address = Subscriber::pda(payer.key(),subscription.key()).0,
    )]
    pub subscriber: Account<'info, Subscriber>,
    #[account(mut)]
    pub subscriber_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        address=Subscription::bank_pda(subscription.key(),subscription.owner.key()).0
    )]
    pub subscription_bank: Account<'info, TokenAccount>,

    #[account(address = Subscription::pda(subscription.owner.key(),subscription.subscription_id.clone()).0)]
    pub subscription: Account<'info, Subscription>,

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
                    SEED_SUBSCRIPTION,
                    subscription.owner.as_ref(),
                    &subscription.subscription_id.to_be_bytes(),
                    &[subscription.bump],
                ]],
            ),
            amount,
        )?;

        Ok(())
    }
}
