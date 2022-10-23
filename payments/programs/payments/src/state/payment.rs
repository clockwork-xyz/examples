use {
    anchor_lang::{prelude::*, AnchorDeserialize},
    std::convert::TryFrom,
};

pub const SEED_PAYMENT: &[u8] = b"payment";

/**
 * Payment
 */

#[account]
#[derive(Debug)]
pub struct Payment {
    pub sender: Pubkey,
    pub recipient: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
}

impl Payment {
    pub fn pubkey(sender: Pubkey, recipient: Pubkey, mint: Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[
                SEED_PAYMENT,
                sender.as_ref(),
                recipient.as_ref(),
                mint.as_ref(),
            ],
            &crate::ID,
        )
        .0
    }
}

impl TryFrom<Vec<u8>> for Payment {
    type Error = Error;
    fn try_from(data: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Payment::try_deserialize(&mut data.as_slice())
    }
}

pub trait PaymentAccount {
    fn new(&mut self, sender: Pubkey, recipient: Pubkey, mint: Pubkey, amount: u64) -> Result<()>;
}

impl PaymentAccount for Account<'_, Payment> {
    fn new(&mut self, sender: Pubkey, recipient: Pubkey, mint: Pubkey, amount: u64) -> Result<()> {
        self.sender = sender;
        self.recipient = recipient;
        self.mint = mint;
        self.amount = amount;
        Ok(())
    }
}
