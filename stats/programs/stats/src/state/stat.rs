use anchor_lang::prelude::*;

pub const SEED_STAT: &[u8] = b"stat";

/**
 * Stat
 */

#[account(zero_copy)]
#[derive(Debug)]
pub struct Stat {
    pub price_feed: Pubkey,
    pub authority: Pubkey,
    pub price_history: [Price; 30000],
    pub lookback_window: i64,
    pub sample_count: i64,
    pub sample_sum: i64,
    pub sample_rate: i64,
    pub twap: i64,
    head: u64,
    tail: u64,
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

    pub fn new(
        &mut self,
        price_feed: Pubkey,
        authority: Pubkey,
        lookback_window: i64,
        sample_rate: i64,
    ) -> Result<()> {
        self.price_feed = price_feed;
        self.authority = authority;
        self.price_history = [Price::default(); 30000];
        self.lookback_window = lookback_window;
        self.sample_count = 0;
        self.sample_sum = 0;
        self.sample_rate = sample_rate;
        self.twap = 0;
        self.head = 0;
        self.tail = 0;
        Ok(())
    }

    fn push(&mut self, price: Price) {
        self.price_history[Stat::index_of(self.head)] = price;

        if Stat::index_of(self.head + 1) == Stat::index_of(self.tail) {
            self.tail.checked_add(1).unwrap();
        }

        self.head.checked_add(1).unwrap();
        self.sample_count.checked_add(1).unwrap();
        self.sample_sum.checked_add(price.price).unwrap();
    }

    fn pop(&mut self) -> Option<Price> {
        if self.sample_count > 0 {
            let popped_element = self.price_history[Stat::index_of(self.tail)];

            self.price_history[Stat::index_of(self.tail)] = Price::default();

            if Stat::index_of(self.tail) == Stat::index_of(self.head) {
                self.head.checked_add(1).unwrap();
            }

            self.tail.checked_add(1).unwrap();
            self.sample_count.checked_sub(1).unwrap();
            self.sample_sum.checked_sub(popped_element.price).unwrap();

            return Some(popped_element);
        }

        None
    }

    fn index_of(counter: u64) -> usize {
        std::convert::TryInto::try_into(counter % 30000).unwrap()
    }

    fn get(&mut self, index: usize) -> Option<Price> {
        if index < self.sample_count as usize {
            None
        } else {
            Some(self.price_history[Stat::index_of(index as u64)])
        }
    }

    pub fn twap(&mut self, timestamp: i64, price: i64) -> Result<()> {
        // always insert first encountered pricing data
        if self.sample_count == 0 {
            self.push(Price { price, timestamp });
        } else {
            let newest_price = self.get(self.head as usize).unwrap();
            let oldest_price = self.get(self.tail as usize).unwrap();

            // if the latest price is after sample rate threshhold then insert new pricing data
            if timestamp >= newest_price.timestamp + self.sample_rate {
                self.push(Price { price, timestamp });
            }

            // while oldest pricing data is less lookback window then pop that element
            while oldest_price.timestamp
                < Clock::get().unwrap().unix_timestamp - self.lookback_window.clone()
            {
                self.pop();
            }

            msg!(
                "     oldest - ts: {}, price: {}",
                oldest_price.timestamp,
                oldest_price.price
            );
            msg!(
                "     newest - ts: {}, price: {}",
                newest_price.timestamp,
                newest_price.price
            );
        }

        match self.sample_sum.checked_div(self.sample_count) {
            Some(twap) => self.twap = twap,
            None => {}
        }

        Ok(())
    }
}

impl TryFrom<Vec<u8>> for Stat {
    type Error = Error;
    fn try_from(data: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Stat::try_deserialize(&mut data.as_slice())
    }
}

#[zero_copy]
#[derive(Default, Debug)]
pub struct Price {
    pub price: i64,
    pub timestamp: i64,
}
