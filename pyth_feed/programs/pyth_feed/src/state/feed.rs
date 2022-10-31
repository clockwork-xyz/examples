use anchor_lang::{prelude::*, AnchorDeserialize};

pub const SEED_FEED: &[u8] = b"feed";

/**
 * Feed
 */

#[account]
#[derive(Debug)]
pub struct Feed {
    pub authority: Pubkey,
    pub pyth_price_feed: Pubkey,
    pub publish_time: i64,
}

impl Feed {
    pub fn pubkey(authority: Pubkey) -> Pubkey {
        Pubkey::find_program_address(&[SEED_FEED, authority.key().as_ref()], &crate::ID).0
    }
}

impl TryFrom<Vec<u8>> for Feed {
    type Error = Error;
    fn try_from(data: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Feed::try_deserialize(&mut data.as_slice())
    }
}

/**
 * FeedAccount
 */

pub trait FeedAccount {
    fn new(&mut self, authority: Pubkey, pyth_price_feed: Pubkey) -> Result<()>;
}

impl FeedAccount for Account<'_, Feed> {
    fn new(&mut self, authority: Pubkey, pyth_price_feed: Pubkey) -> Result<()> {
        self.authority = authority;
        self.pyth_price_feed = pyth_price_feed;
        self.publish_time = 0;
        Ok(())
    }
}
