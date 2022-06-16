use {
    crate::pda::PDA,
    anchor_lang::{prelude::*, AnchorDeserialize},
    std::convert::TryFrom,
};

pub const SEED_ESCROW: &[u8] = b"escrow";

/**
 * Escrow
 */

#[account]
#[derive(Debug)]
pub struct Escrow {
    pub amount: u64,
    pub mint: Pubkey,
    pub recipient: Pubkey,
    pub sender: Pubkey,
    pub transfer_rate: u64,
}

impl Escrow {
    pub fn pda() -> PDA {
        Pubkey::find_program_address(&[SEED_ESCROW], &crate::ID)
    }
}

impl TryFrom<Vec<u8>> for Escrow {
    type Error = Error;
    fn try_from(data: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Escrow::try_deserialize(&mut data.as_slice())
    }
}

pub trait EscrowAccount {
    fn new(
        &mut self,
        amount: u64,
        mint: Pubkey,
        recipient: Pubkey,
        sender: Pubkey,
        transfer_rate: u64,
    ) -> Result<()>;
}

impl EscrowAccount for Account<'_, Escrow> {
    fn new(
        &mut self,
        amount: u64,
        mint: Pubkey,
        recipient: Pubkey,
        sender: Pubkey,
        transfer_rate: u64,
    ) -> Result<()> {
        self.amount = amount;
        self.mint = mint;
        self.recipient = recipient;
        self.sender = sender;
        self.transfer_rate = transfer_rate;
        Ok(())
    }
}
