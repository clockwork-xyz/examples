use {
    crate::state::*,
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{self, TokenAccount, Transfer},
    },
    clockwork_crank::state::{CrankResponse, Queue, SEED_QUEUE},
};

#[derive(Accounts)]
pub struct DisbursePayment<'info> {
    #[account(
        mut,
        address = Subscriber::pubkey(subscriber.owner.key(),subscription.key()),
    )]
    pub subscriber: Account<'info, Subscriber>,
    #[account(
        mut,
        associated_token::authority = subscription,
        associated_token::mint = subscription.mint,
    )]
    pub subscription_bank: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        address = Subscription::pubkey(subscription.owner,subscription.subscription_id.clone())
    )]
    pub subscription: Box<Account<'info, Subscription>>,
    #[account(
        signer,
        seeds = [
            SEED_QUEUE,
            subscription.key().as_ref(),
            "subscription".as_bytes()
        ],
        seeds::program = clockwork_crank::ID,
        bump,
    )]
    pub subscription_queue: Box<Account<'info, Queue>>,

    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

impl<'info> DisbursePayment<'_> {
    pub fn process(&mut self, bump: u8) -> Result<CrankResponse> {
        let Self {
            subscriber,
            subscription,
            ..
        } = self;

        if subscriber.locked_amount - subscription.recurrent_amount < 0 {
            subscriber.is_active = false;
            subscriber.is_subscribed = false
        } else {
            subscriber.locked_amount -= subscription.recurrent_amount;
            subscriber.is_subscribed = true
        }

        // transfer from escrow to recipient's token account
        // token::transfer(
        //     CpiContext::new_with_signer(
        //         token_program.to_account_info(),
        //         Transfer {
        //             from: escrow.to_account_info(),
        //             to: subscription_bank.to_account_info(),
        //             authority: subscription.to_account_info(),
        //         },
        //         &[&[SEED_SUBSCRIPTION, subscription.owner.as_ref(), subscription.subscription_id.as_bytes(), &[bump]]]),
        //     subscription.recurrent_amount,
        // )?;

        Ok(CrankResponse {
            next_instruction: None,
        })
    }
}
