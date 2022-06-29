use {
    crate::pda::PDA,
    anchor_lang::{prelude::*, AnchorDeserialize},
    anchor_spl::dex::serum_dex::matching::Side,
    std::convert::TryFrom,
};

pub const SEED_ORDER: &[u8] = b"order";

/**
 * Order
 */

#[account]
#[derive(Debug)]
pub struct Order {
    pub fund: Pubkey,
    pub buyer: Pubkey,
    pub amount: u64,
    pub side: Side,
    pub supply_snapshot: u64,
    pub usdc_slippage_refunded: u64,
    pub asset_index: u8,
}

impl Order {
    pub fn pda(fund: Pubkey, buyer: Pubkey) -> PDA {
        Pubkey::find_program_address(
            &[SEED_ORDER, fund.key().as_ref(), buyer.key().as_ref()],
            &crate::ID,
        )
    }
}

impl TryFrom<Vec<u8>> for Order {
    type Error = Error;
    fn try_from(data: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Order::try_deserialize(&mut data.as_slice())
    }
}

/**
 * OrderAccount
 */

pub trait OrderAccount {
    fn new(
        &mut self,
        fund: Pubkey,
        buyer: Pubkey,
        amount: u64,
        side: Side,
        supply_snapshot: u64,
        usdc_slippage_refunded: u64,
        asset_index: u8,
    ) -> Result<()>;
}

impl OrderAccount for Account<'_, Order> {
    fn new(
        &mut self,
        fund: Pubkey,
        buyer: Pubkey,
        amount: u64,
        side: Side,
        supply_snapshot: u64,
        usdc_slippage_refunded: u64,
        asset_index: u8,
    ) -> Result<()> {
        self.fund = fund;
        self.buyer = buyer;
        self.amount = amount;
        self.side = side;
        self.supply_snapshot = supply_snapshot;
        self.usdc_slippage_refunded = usdc_slippage_refunded;
        self.asset_index = asset_index;
        Ok(())
    }
}
