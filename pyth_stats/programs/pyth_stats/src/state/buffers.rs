use anchor_lang::prelude::*;

pub const SEED_TIME_SERIES: &[u8] = b"time_series";

/**
 * TimeSeries
 */

#[account(zero_copy)]
pub struct TimeSeries {}

impl TimeSeries {
    pub fn pubkey(stat: Pubkey) -> Pubkey {
        Pubkey::find_program_address(&[SEED_TIME_SERIES, stat.as_ref()], &crate::ID).0
    }
}

pub const SEED_PRICE_BUFFER: &[u8] = b"price_buffer";

/**
 * PriceBuffer
 */

#[account(zero_copy)]
pub struct PriceBuffer {}

impl PriceBuffer {
    pub fn pubkey(stat: Pubkey) -> Pubkey {
        Pubkey::find_program_address(&[SEED_PRICE_BUFFER, stat.as_ref()], &crate::ID).0
    }
}

pub const SEED_AVG_BUFFER: &[u8] = b"avg_buffer";

/**
 * AvgBuffer
 */

#[account(zero_copy)]
pub struct AvgBuffer {}

impl AvgBuffer {
    pub fn pubkey(stat: Pubkey) -> Pubkey {
        Pubkey::find_program_address(&[SEED_AVG_BUFFER, stat.as_ref()], &crate::ID).0
    }
}
