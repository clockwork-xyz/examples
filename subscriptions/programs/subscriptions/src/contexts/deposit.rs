use {
    crate::{error::ErrorCode, state::*},
    anchor_lang::prelude::*,
    anchor_spl::token::{transfer, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct Deposit<'info> {
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

impl<'info> Deposit<'_> {
    pub fn process(&mut self, amount: u64) -> Result<()> {
        let Self {
            payer,
            subscriber,
            subscriber_token_account,
            subscription_bank,
            subscription,
            token_program,
            ..
        } = self;
        require!(subscription.is_active, ErrorCode::SubscriptionInactive);

        transfer(
            CpiContext::new(
                token_program.to_account_info(),
                Transfer {
                    from: subscriber_token_account.to_account_info(),
                    to: subscription_bank.to_account_info(),
                    authority: payer.to_account_info(),
                },
            ),
            amount,
        )?;

        subscriber.locked_amount += amount;

        Ok(())
    }
}
