use anchor_lang::prelude::*;

#[derive(Debug, Clone)]
pub struct OpenBookDex;

impl Id for OpenBookDex {
    fn id() -> Pubkey {
        #[cfg(feature = "localnet")]
        use std::str::FromStr;
        #[cfg(feature = "localnet")]
        return Pubkey::from_str("6QUbdf53eJZToaiLBrWbsJCE8jhXYAvufuJ4rzRskiCJ").unwrap();

        #[cfg(not(feature = "localnet"))]
        anchor_spl::dex::ID
        // TODO: the devnet flag is not propagated to anchor_spl! So this will be mainnet ID!

        // ORIGINAL OPENBOOK DEX CODE BELOW:
        // #[cfg(not(feature = "devnet"))]
        // anchor_lang::solana_program::declare_id!("srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX");
        //
        // #[cfg(feature = "devnet")]
        // anchor_lang::solana_program::declare_id!("EoTcMgcDRTJVZDMZWBoU6rhYHZfkNTVEAfz3uUJRcYGj");
    }
}
