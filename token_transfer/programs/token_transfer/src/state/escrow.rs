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
    pub mint: Pubkey,
    pub recipient: Pubkey,
    pub sender: Pubkey,
    pub queue_pubkey: Option<Pubkey>,
    pub amount: u64,
    pub transfer_rate: u64,
}

impl Escrow {
    pub fn pda(sender: Pubkey, recipient: Pubkey) -> PDA {
        Pubkey::find_program_address(
            &[SEED_ESCROW, sender.as_ref(), recipient.as_ref()],
            &crate::ID,
        )
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
        mint: Pubkey,
        recipient: Pubkey,
        sender: Pubkey,
        queue_pubkey: Option<Pubkey>,
        amount: u64,
        transfer_rate: u64,
    ) -> Result<()>;
}

impl EscrowAccount for Account<'_, Escrow> {
    fn new(
        &mut self,
        mint: Pubkey,
        recipient: Pubkey,
        sender: Pubkey,
        queue_pubkey: Option<Pubkey>,
        amount: u64,
        transfer_rate: u64,
    ) -> Result<()> {
        self.mint = mint;
        self.recipient = recipient;
        self.sender = sender;
        self.queue_pubkey = queue_pubkey;
        self.amount = amount;
        self.transfer_rate = transfer_rate;
        Ok(())
    }
}
