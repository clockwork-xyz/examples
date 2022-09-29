use {
    anchor_lang::{prelude::*, AnchorDeserialize},
    std::convert::TryFrom,
};

pub const SEED_EVENT: &[u8] = b"event";

/**
 * Fee
 */

#[account]
#[derive(Debug)]
pub struct Event {
    pub timestamp: i64,
    pub user: Pubkey,
}

impl Event {
    pub fn pubkey() -> Pubkey {
        Pubkey::find_program_address(&[SEED_EVENT], &crate::ID).0
    }
}

impl TryFrom<Vec<u8>> for Event {
    type Error = Error;
    fn try_from(data: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Event::try_deserialize(&mut data.as_slice())
    }
}
