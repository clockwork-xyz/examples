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
    price_history: HashMap<u64, i64>,
    lookback_window: u64,
    twap: i64,
}

impl Stat {
    pub fn pubkey() -> Pubkey {
        Pubkey::find_program_address(&[SEED_STAT], &crate::ID).0
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
    fn new(&mut self, lookback_window: u64) -> Result<()>;
    fn twap(&mut self, price: u64, timestamp: u64) -> Result<()>;
}

impl StatAccount for Account<'_, Stat> {
    fn new(&mut self, lookback_window: u64) -> Result<()> {
        self.price_history = HashMap::new();
        self.lookback_window = lookback_window;
        self.twap = 0;
        Ok(())
    }

    fn twap(&mut self, price: u64, timestamp: u64) -> Result<()> {
        //   TODO:
        // - index new price value into dashmap
        // self.price_history

        // - shave dashmap to only allow values within the lookback window
        // - calculate TWAP

        Ok(())
    }
}
