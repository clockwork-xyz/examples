use anchor_lang::prelude::*;

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
