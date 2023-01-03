use anchor_lang::prelude::*;

#[derive(Debug, Clone)]
pub struct OpenBookDex;

impl Id for OpenBookDex {
    fn id() -> Pubkey {
        #[cfg(feature = "localnet")]
        use std::str::FromStr;
        #[cfg(feature = "localnet")]
        // the program id of the openbook dex program as found in dex/serum_dex-keypair.json
        return Pubkey::from_str("6QUbdf53eJZToaiLBrWbsJCE8jhXYAvufuJ4rzRskiCJ").unwrap();

        #[cfg(not(feature = "localnet"))]
        anchor_spl::dex::ID

        // ORIGINAL OPENBOOK DEX CODE BELOW:
        // #[cfg(not(feature = "devnet"))]
        // anchor_lang::solana_program::declare_id!("srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX");
        //
        // #[cfg(feature = "devnet")]
        // anchor_lang::solana_program::declare_id!("EoTcMgcDRTJVZDMZWBoU6rhYHZfkNTVEAfz3uUJRcYGj");
    }
}
