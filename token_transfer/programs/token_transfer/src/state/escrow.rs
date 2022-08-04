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
    pub sender: Pubkey,
    pub recipient: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub transfer_rate: u64,
    pub queue: Option<Pubkey>,
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
        sender: Pubkey,
        recipient: Pubkey,
        mint: Pubkey,
        amount: u64,
        transfer_rate: u64,
        queue: Option<Pubkey>,
    ) -> Result<()>;
}

impl EscrowAccount for Account<'_, Escrow> {
    fn new(
        &mut self,
        sender: Pubkey,
        recipient: Pubkey,
        mint: Pubkey,
        amount: u64,
        transfer_rate: u64,
        queue: Option<Pubkey>,
    ) -> Result<()> {
        self.sender = sender;
        self.recipient = recipient;
        self.mint = mint;
        self.amount = amount;
        self.transfer_rate = transfer_rate;
        self.queue = queue;
        Ok(())
    }
}
