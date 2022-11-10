use {
    anchor_lang::{prelude::*, AnchorDeserialize},
    std::collections::HashMap,
};

pub const SEED_STAT: &[u8] = b"stat";

/**
 * Stat
 */

#[account]
#[derive(Debug)]
pub struct Stat {
    pub price_feed: Pubkey,
    pub authority: Pubkey,
    pub price_history: HashMap<i64, i64>,
    pub lookback_window: i64,
    pub twap: i64,
}

impl Stat {
    pub fn pubkey(price_feed: Pubkey, authority: Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[SEED_STAT, price_feed.as_ref(), authority.as_ref()],
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

pub trait StatAccount {
    fn new(&mut self, price_feed: Pubkey, authority: Pubkey, lookback_window: i64) -> Result<()>;
    fn twap(&mut self, timestamp: i64, price: i64) -> Result<()>;
}

impl StatAccount for Account<'_, Stat> {
    fn new(&mut self, price_feed: Pubkey, authority: Pubkey, lookback_window: i64) -> Result<()> {
        self.price_feed = price_feed;
        self.authority = authority;
        self.price_history = HashMap::new();
        self.lookback_window = lookback_window;
        self.twap = 0;
        Ok(())
    }

    fn twap(&mut self, timestamp: i64, price: i64) -> Result<()> {
        let lookback_window = self.lookback_window.clone();
        // - index new price value into dashmap
        self.price_history.insert(timestamp, price);
        // - retain prices only within the lookback window
        self.price_history
            .retain(|&k, _| k > Clock::get().unwrap().unix_timestamp - lookback_window);

        let len = self.price_history.len();
        let sum: i64 = self.price_history.values().sum();
        self.twap = sum.saturating_div(len as i64);

        Ok(())
    }
}
