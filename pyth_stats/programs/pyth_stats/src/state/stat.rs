use anchor_lang::prelude::*;

pub const SEED_STAT: &[u8] = b"stat";
/**
 * Stat
 */

#[account()]
pub struct Stat {
    pub price_feed: Pubkey,
    pub authority: Pubkey,
    pub lookback_window: i64,
    pub sample_count: usize,
    pub sample_sum: i64,
    pub buffer_size: usize,
    pub head: Option<i64>,
}

impl Stat {
    pub fn pubkey(price_feed: Pubkey, authority: Pubkey, lookback_window: i64) -> Pubkey {
        Pubkey::find_program_address(
            &[
                SEED_STAT,
                price_feed.as_ref(),
                authority.as_ref(),
                &lookback_window.to_le_bytes(),
            ],
            &crate::ID,
        )
        .0
    }
}

impl TryFrom<Vec<u8>> for Stat {
    type Error = Error;
    fn try_from(data: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Stat::try_deserialize(&mut data.as_slice())
    }
}

/**
 * StatAccount
 */

impl Stat {
    pub fn new(
        &mut self,
        price_feed: Pubkey,
        authority: Pubkey,
        lookback_window: i64,
        buffer_size: usize,
    ) -> Result<()> {
        self.price_feed = price_feed;
        self.authority = authority;
        self.lookback_window = lookback_window;
        self.sample_count = 0;
        self.sample_sum = 0;
        self.buffer_size = buffer_size;
        self.head = None;
        Ok(())
    }
}
