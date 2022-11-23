use {
    anchor_lang::{prelude::*, AnchorDeserialize},
    std::convert::TryFrom,
};

pub const SEED_SUBSCRIBER: &[u8] = b"subscriber";

/**
 * Subscriber
 */

#[account]
#[derive(Debug)]
pub struct Subscriber {
    pub owner: Pubkey,
    pub subscription: Pubkey,
    pub locked_amount: u64,
    pub is_subscribed: bool,
    pub is_active: bool,
}

impl Subscriber {
    pub fn pda(owner: Pubkey, subscription: Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[SEED_SUBSCRIBER, owner.as_ref(), subscription.as_ref()],
            &crate::ID,
        )
    }
}

impl TryFrom<Vec<u8>> for Subscriber {
    type Error = Error;
    fn try_from(data: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Subscriber::try_deserialize(&mut data.as_slice())
    }
}

pub trait SubscriberAccount {
    fn new(
        &mut self,
        owner: Pubkey,
        subscription: Pubkey,
        locked_amount: u64,
        is_subscribed: bool,
        is_active: bool,
    ) -> Result<()>;
}

impl SubscriberAccount for Account<'_, Subscriber> {
    fn new(
        &mut self,
        owner: Pubkey,
        subscription: Pubkey,
        locked_amount: u64,
        is_subscribed: bool,
        is_active: bool,
    ) -> Result<()> {
        self.owner = owner;
        self.subscription = subscription;
        self.locked_amount = locked_amount;
        self.is_subscribed = is_subscribed;
        self.is_active = is_active;
        Ok(())
    }
}
