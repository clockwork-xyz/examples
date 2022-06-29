use {
    crate::pda::PDA,
    anchor_lang::{prelude::*, AnchorDeserialize},
    serum_swap::Side,
    std::convert::TryFrom,
};

pub const SEED_FUND: &[u8] = b"fund";

/**
 * Fund
 */

#[account]
#[derive(Debug)]
pub struct Fund {
    pub name: String,
    pub symbol: String,
    pub manager: Pubkey,
    pub assets: [Pubkey; 3],
    pub weights: [u64; 3],
    pub index_token_mint: Pubkey,
}

impl Fund {
    pub fn pda(manager: Pubkey, name: String) -> PDA {
        Pubkey::find_program_address(&[SEED_FUND, manager.as_ref(), name.as_ref()], &crate::ID)
    }
}

impl TryFrom<Vec<u8>> for Fund {
    type Error = Error;
    fn try_from(data: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Fund::try_deserialize(&mut data.as_slice())
    }
}

pub trait FundAccount {
    fn new(
        &mut self,
        name: String,
        symbol: String,
        manager: Pubkey,
        assets: [Pubkey; 3],
        weights: [u64; 3],
        index_token_mint: Pubkey,
    ) -> Result<()>;
}

impl FundAccount for Account<'_, Fund> {
    fn new(
        &mut self,
        name: String,
        symbol: String,
        manager: Pubkey,
        assets: [Pubkey; 3],
        weights: [u64; 3],
        index_token_mint: Pubkey,
    ) -> Result<()> {
        self.name = name;
        self.symbol = symbol;
        self.manager = manager;
        self.assets = assets;
        self.weights = weights;
        self.index_token_mint = index_token_mint;
        Ok(())
    }
}

// pub payer: Pubkey,
// pub amount: u64,
// pub swap_rate: u64,
// pub from_token: Pubkey,
// pub to_token: Pubkey,
// pub side: Side,
// pub queue_pubkey: Option<Pubkey>,
