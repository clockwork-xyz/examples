use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Insuffiscient amount to transfer")]
    InsuffiscientAmount,
    #[msg("Subscription is inactive")]
    SubscriptionInactive,
    #[msg("payer is not the owner of the subscription")]
    NotOwner,
}
