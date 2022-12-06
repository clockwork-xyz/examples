use anchor_lang::prelude::*;

pub const SEED_STAT: &[u8] = b"stat";

pub const PRICE_ARRAY_SIZE: usize = 5;

/**
 * Stat
 */

#[account(zero_copy)]
pub struct Stat {
    pub price_feed: Pubkey,                       // 32
    pub authority: Pubkey,                        // 32
    pub price_history: [Price; PRICE_ARRAY_SIZE], // 16 * 655_347
    pub lookback_window: i64,                     // 8
    pub sample_count: i64,                        // 8
    pub sample_sum: i64,                          // 8
    pub sample_rate: i64,                         // 8
    pub twap: i64,                                // 8
    pub head: u64,                                // 8
    pub tail: u64,                                // 8
                                                  // = 32 + 32 + (16 * 655,347) + 8 + 8 + 8 + 8 + 8 + 8 + 8   = ~10,485,680 bytes
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
        self.price_history = [Price::default(); PRICE_ARRAY_SIZE];
        self.lookback_window = lookback_window;
        self.sample_count = 0;
        self.sample_sum = 0;
        self.sample_rate = sample_rate;
        self.twap = 0;
        self.head = 0;
        self.tail = 0;
        Ok(())
    }

    fn push(&mut self, price: Price) -> Result<()> {
        // if array is full pop element to avoid overflow
        if Stat::index_of(self.head + 1) == Stat::index_of(self.tail) {
            self.pop()?;
        }

        if Stat::index_of(self.head + 1) == 0 {
            self.head = 0;
        } else if self.sample_count > 0 {
            self.head = self.head.checked_add(1).unwrap();
        }

        self.price_history[Stat::index_of(self.head)] = price;

        self.sample_count = self.sample_count.checked_add(1).unwrap();
        self.sample_sum = self.sample_sum.checked_add(price.price).unwrap();

        Ok(())
    }

    fn pop(&mut self) -> Result<Option<Price>> {
        if self.sample_count > 0 {
            let popped_element = self.price_history[Stat::index_of(self.tail)];

            self.price_history[Stat::index_of(self.tail)] = Price::default();

            if Stat::index_of(self.tail + 1) == 0 {
                self.tail = 0;
            } else {
                self.tail = self.tail.checked_add(1).unwrap();
            }

            self.sample_count = self.sample_count.checked_sub(1).unwrap();
            self.sample_sum = self.sample_sum.checked_sub(popped_element.price).unwrap();

            return Ok(Some(popped_element));
        }

        Ok(None)
    }

    pub fn index_of(counter: u64) -> usize {
        std::convert::TryInto::try_into(counter % PRICE_ARRAY_SIZE as u64).unwrap()
    }

    pub fn twap(&mut self, price: Price) -> Result<()> {
        // always insert first encountered pricing data
        if self.sample_count == 0 {
            self.push(price)?;
        } else {
            let newest_price = self.price_history[Stat::index_of(self.head as u64)];
            let oldest_price = self.price_history[Stat::index_of(self.tail as u64)];

            // if the latest price is after sample rate threshhold then insert new pricing data
            if price.timestamp >= newest_price.timestamp + self.sample_rate {
                self.push(price)?;
            }

            // while oldest pricing data is less lookback window then pop that element
            while oldest_price.timestamp
                < Clock::get().unwrap().unix_timestamp - self.lookback_window.clone()
            {
                let _popped_element = self.pop()?;
            }
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
    pub price: i64,     // 8
    pub timestamp: i64, // 8
}
