use {
    anchor_lang::{prelude::*, AnchorDeserialize},
    std::collections::VecDeque,
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
    pub price_history: VecDeque<(i64, i64)>,
    pub lookback_window: i64,
    pub sample_count: i64,
    pub sample_sum: i64,
    pub sample_rate: i64,
    pub twap: i64,
    pub id: String,
}

impl Stat {
    pub fn pubkey(price_feed: Pubkey, authority: Pubkey, id: String) -> Pubkey {
        Pubkey::find_program_address(
            &[
                SEED_STAT,
                price_feed.as_ref(),
                authority.as_ref(),
                id.as_bytes(),
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

pub trait StatAccount {
    fn new(
        &mut self,
        price_feed: Pubkey,
        authority: Pubkey,
        lookback_window: i64,
        sample_rate: i64,
        id: String,
    ) -> Result<()>;
    fn twap(&mut self, timestamp: i64, price: i64) -> Result<()>;
}

impl StatAccount for Account<'_, Stat> {
    fn new(
        &mut self,
        price_feed: Pubkey,
        authority: Pubkey,
        lookback_window: i64,
        sample_rate: i64,
        id: String,
    ) -> Result<()> {
        self.price_feed = price_feed;
        self.authority = authority;
        self.price_history = VecDeque::new();
        self.lookback_window = lookback_window;
        self.sample_count = 0;
        self.sample_sum = 0;
        self.sample_rate = sample_rate;
        self.twap = 0;
        self.id = id;
        Ok(())
    }

    fn twap(&mut self, timestamp: i64, price: i64) -> Result<()> {
        // always insert first encountered pricing data
        if self.sample_count == 0 {
            self.price_history.push_back((timestamp, price));
            self.sample_sum = self.sample_sum.checked_add(price).unwrap();
            self.sample_count = self.sample_count.checked_add(1).unwrap();
        } else {
            let newest_price = *self
                .price_history
                .get((self.sample_count - 1) as usize)
                .unwrap();
            let oldest_price = *self.price_history.get(0).unwrap();

            // if the latest price is after sample rate threshhold then insert new pricing data
            if timestamp >= newest_price.0 + self.sample_rate {
                self.price_history.push_back((timestamp, price));
                self.sample_sum = self.sample_sum.checked_add(price).unwrap();
                self.sample_count = self.sample_count.checked_add(1).unwrap();
            }

            // while oldest pricing data is less lookback window then pop that element
            while oldest_price.0
                < Clock::get().unwrap().unix_timestamp - self.lookback_window.clone()
            {
                let popped_element = self.price_history.pop_front().unwrap();
                self.sample_sum = self.sample_sum.checked_sub(popped_element.1).unwrap();
                self.sample_count = self.sample_count.checked_sub(1).unwrap();
            }

            msg!(
                "     oldest - ts: {}, price: {}",
                oldest_price.0,
                oldest_price.1
            );
            msg!(
                "     newest - ts: {}, price: {}",
                newest_price.0,
                newest_price.1
            );
        }

        match self.sample_sum.checked_div(self.sample_count) {
            Some(twap) => self.twap = twap,
            None => {}
        }

        Ok(())
    }
}
