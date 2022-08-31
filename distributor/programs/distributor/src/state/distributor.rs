use anchor_lang::{prelude::*, AnchorDeserialize};

pub const SEED_DISTRIBUTOR: &[u8] = b"distributor";

/**
 * Distributor
 */

#[account]
#[derive(Debug)]
pub struct Distributor {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub recipient: Pubkey,
}

impl Distributor {
    pub fn pubkey(mint: Pubkey, admin: Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[SEED_DISTRIBUTOR, mint.as_ref(), admin.as_ref()],
            &crate::ID,
        )
        .0
    }
}

impl TryFrom<Vec<u8>> for Distributor {
    type Error = Error;
    fn try_from(data: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Distributor::try_deserialize(&mut data.as_slice())
    }
}

/**
 * DistributorAccount
 */

pub trait DistributorAccount {
    fn new(&mut self, admin: Pubkey, recipient: Pubkey, mint: Pubkey) -> Result<()>;
}

impl DistributorAccount for Account<'_, Distributor> {
    fn new(&mut self, admin: Pubkey, recipient: Pubkey, mint: Pubkey) -> Result<()> {
        self.admin = admin;
        self.recipient = recipient;
        self.mint = mint;
        Ok(())
    }
}
