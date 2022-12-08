use bytemuck::{Pod, Zeroable};

use {
    anchor_lang::{
        prelude::{borsh::BorshSchema, *},
        Discriminator,
    },
    arrayref::array_ref,
    std::{
        cell::{Ref, RefMut},
        mem,
    },
};

pub const SEED_STAT: &[u8] = b"stat";

pub const PRICE_ARRAY_SIZE: usize = 5;

/**
 * Stat
 */

#[account(zero_copy)]
pub struct Stat {
    pub price_feed: Pubkey,   // 32
    pub authority: Pubkey,    // 32
    pub lookback_window: i64, // 8
    pub sample_count: i64,    // 8
    pub sample_sum: i64,      // 8
    pub sample_rate: i64,     // 8
    pub twap: i64,            // 8
    pub head: u64,            // 8
    pub tail: u64,            // 8
                              // = 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 8 = 120 bytes
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
        self.lookback_window = lookback_window;
        self.sample_count = 0;
        self.sample_sum = 0;
        self.sample_rate = sample_rate;
        self.twap = 0;
        self.head = 0;
        self.tail = 0;
        Ok(())
    }

    fn push(&mut self, price: Price, price_history: &mut RefMut<[Price]>) -> Result<()> {
        // if array is full pop element to avoid overflow
        if Stat::index_of(self.head + 1) == Stat::index_of(self.tail) {
            self.pop(price_history)?;
        }

        if Stat::index_of(self.head + 1) == 0 {
            self.head = 0;
        } else if self.sample_count > 0 {
            self.head = self.head.checked_add(1).unwrap();
        }

        price_history[Stat::index_of(self.head)] = price;

        self.sample_count = self.sample_count.checked_add(1).unwrap();
        self.sample_sum = self.sample_sum.checked_add(price.price).unwrap();

        Ok(())
    }

    fn pop(&mut self, price_history: &mut RefMut<[Price]>) -> Result<Option<Price>> {
        if self.sample_count > 0 {
            let popped_element = price_history[Stat::index_of(self.tail)];

            price_history[Stat::index_of(self.tail)] = Price::default();

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

    pub fn twap(&mut self, price: Price, price_history: &mut RefMut<[Price]>) -> Result<()> {
        // always insert first encountered pricing data
        if self.sample_count == 0 {
            self.push(price, price_history)?;
        } else {
            let newest_price = price_history[Stat::index_of(self.head as u64)];
            let oldest_price = price_history[Stat::index_of(self.tail as u64)];

            // if the latest price is after sample rate threshhold then insert new pricing data
            if price.timestamp >= newest_price.timestamp + self.sample_rate {
                self.push(price, price_history)?;
            }

            // while oldest pricing data is less lookback window then pop that element
            while oldest_price.timestamp
                < Clock::get().unwrap().unix_timestamp - self.lookback_window.clone()
            {
                let _popped_element = self.pop(price_history)?;
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
#[derive(AnchorSerialize, AnchorDeserialize, BorshSchema, Default, Pod, Zeroable)]
pub struct Price {
    pub price: i64,     // 8
    pub timestamp: i64, // 8
}

/// Returns a Ref to a slice of Price entries, after the discriminator and header
#[inline(always)]
pub fn load_entries<'a, THeader, TEntries>(data: Ref<'a, &mut [u8]>) -> Result<Ref<'a, [TEntries]>>
where
    THeader: Discriminator,
    TEntries: bytemuck::Pod + Copy,
{
    if data.len() < THeader::discriminator().len() {
        return err!(ErrorCode::AccountDiscriminatorNotFound);
    }
    let disc_bytes: &[u8; 8] = array_ref![data, 0, 8];
    if disc_bytes != &THeader::discriminator() {
        return err!(ErrorCode::AccountDiscriminatorMismatch);
    }

    Ok(Ref::map(data, |data| {
        bytemuck::cast_slice(&data[8 + mem::size_of::<THeader>()..data.len()])
    }))
}

/// Returns a mutable Ref to a slice of Price entries, after the discriminator and header
#[inline(always)]
pub fn load_entries_mut<'a, THeader, TEntries>(
    data: RefMut<'a, &mut [u8]>,
) -> Result<RefMut<'a, [TEntries]>>
where
    THeader: Discriminator,
    TEntries: bytemuck::Pod + Copy,
{
    if data.len() < THeader::discriminator().len() {
        return err!(ErrorCode::AccountDiscriminatorNotFound);
    }
    let disc_bytes: &[u8; 8] = array_ref![data, 0, 8];
    if disc_bytes != &THeader::discriminator() {
        return err!(ErrorCode::AccountDiscriminatorMismatch);
    }

    Ok(RefMut::map(data, |data| {
        let len = data.len();
        bytemuck::cast_slice_mut::<u8, TEntries>(&mut data[8 + mem::size_of::<THeader>()..len])
    }))
}
