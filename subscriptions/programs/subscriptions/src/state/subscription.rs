use {
    anchor_lang::{prelude::*, AnchorDeserialize},
    std::convert::TryFrom,
};

pub const SEED_SUBSCRIPTION: &[u8] = b"subscription";

/**
 * Subscription
 */

#[account]
#[derive(Debug)]
pub struct Subscription {
    pub recipient: Pubkey,
    pub mint: Pubkey,
    pub recurrent_amount: u64,
    pub epochs_reset:u64,
    pub schedule: String,
}

impl Subscription {
    pub fn pubkey(sender: Pubkey, recipient: Pubkey, mint: Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[
                SEED_SUBSCRIPTION,
                sender.as_ref(),
                recipient.as_ref(),
                mint.as_ref(),
            ],
            &crate::ID,
        )
        .0
    }
}

impl TryFrom<Vec<u8>> for Subscription {
    type Error = Error;
    fn try_from(data: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Subscription::try_deserialize(&mut data.as_slice())
    }
}

pub trait SubscriptionAccount {
    fn new(
        &mut self,
        recipient: Pubkey,
        mint:Pubkey,
        recurrent_amount: u64,
        epochs_reset:u64,
        schedule: String,
    ) -> Result<()>;
}

impl SubscriptionAccount for Account<'_, Subscription> {
    fn new(
        &mut self,
        recipient: Pubkey,
        mint: Pubkey,
        recurrent_amount: u64,
        epochs_reset: u64,
        schedule: String,
    ) -> Result<()> {
        self.recipient = recipient;
        self.mint = mint;
        self.recurrent_amount = recurrent_amount;
        self.epochs_reset = epochs_reset;
        self.schedule = schedule;
        Ok(())
    }
}
