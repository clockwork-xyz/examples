use anchor_lang::prelude::*;

#[derive(Debug, Clone)]
pub struct OpenBookDex;

impl Id for OpenBookDex {
    fn id() -> Pubkey {
        anchor_spl::dex::ID

        // ORIGINAL OPENBOOK DEX CODE BELOW:
        // #[cfg(not(feature = "devnet"))]
        // anchor_lang::solana_program::declare_id!("srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX");
        //
        // #[cfg(feature = "devnet")]
        // anchor_lang::solana_program::declare_id!("EoTcMgcDRTJVZDMZWBoU6rhYHZfkNTVEAfz3uUJRcYGj");
    }
}
