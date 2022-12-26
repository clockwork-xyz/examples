use {
    anchor_lang::prelude::*,
    bytemuck::{Pod, Zeroable},
    pyth_sdk_solana::Price,
};

pub const SEED_DATASET: &[u8] = b"dataset";

/**
 * Dataset
 */

#[account(zero_copy)]
pub struct Dataset {}

impl Dataset {
    pub fn pubkey(stat: Pubkey) -> Pubkey {
        Pubkey::find_program_address(&[SEED_DATASET, stat.as_ref()], &crate::ID).0
    }
}

#[derive(Copy, Clone, Zeroable, Pod, Default)]
#[repr(C)]
pub struct PriceData {
    pub price: i64,
    pub ts: i64,
}

impl From<Price> for PriceData {
    fn from(price: Price) -> Self {
        PriceData {
            price: price.price,
            ts: price.publish_time,
        }
    }
}
