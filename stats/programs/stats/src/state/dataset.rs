use anchor_lang::prelude::*;

pub const SEED_DATASET: &[u8] = b"dataset";

/**
 * Dataset
 */

#[account(zero_copy)]
pub struct Dataset {}

impl Dataset {
    pub fn pubkey(stats: Pubkey) -> Pubkey {
        Pubkey::find_program_address(&[SEED_DATASET, stats.as_ref()], &crate::ID).0
    }
}
