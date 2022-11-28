use {
    crate::{error::ErrorCode, state::*},
    anchor_lang::prelude::*,
    anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct Subscribe<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        address = Subscriber::pda(payer.key(),subscription.key()).0,
    )]
    pub subscriber: Account<'info, Subscriber>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = payer
    )]
    pub subscriber_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = subscription,
        address = Subscription::bank_pda(subscription.key(),subscription.owner.key()).0
    )]
    pub subscription_bank: Account<'info, TokenAccount>,
    #[account(address=subscription.mint)]
    pub mint: Account<'info, Mint>,

    #[account(mut, address = Subscription::pda(subscription.owner.key(),subscription.subscription_id.clone()).0)]
    pub subscription: Account<'info, Subscription>,
    pub token_program: Program<'info, Token>,
}

impl<'info> Subscribe<'_> {
    pub fn process(&mut self) -> Result<()> {
        let Self {
            payer,
            subscriber,
            subscription,
            subscriber_token_account,
            token_program,
            subscription_bank,
            ..
        } = self;

        require!(
            subscriber_token_account.amount >= subscription.recurrent_amount,
            ErrorCode::InsuffiscientAmountLocked
        );
        require!(subscription.is_active, ErrorCode::SubscriptionInactive);

        token::transfer(
            CpiContext::new(
                token_program.to_account_info(),
                Transfer {
                    authority: payer.to_account_info(),
                    from: subscriber_token_account.to_account_info(),
                    to: subscription_bank.to_account_info(),
                },
            ),
            subscription.recurrent_amount,
        )?;

        subscriber.is_active = true;
        subscriber.is_subscribed = true;
        Ok(())
    }
}
