use crate::id;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

pub const SETTINGS_SEED: &str = "settings_seed";
pub const TOKEN_SEED: &str = "token_seed";
pub const LAMPORTS_SEED: &str = "lamports_seed";

#[derive(BorshDeserialize, BorshSerialize)]
pub struct MarketSettings {
    pub admin: Pubkey,
    pub sell_price: u64,
    pub buy_price: u64,
    pub mint: Pubkey,
}

impl MarketSettings {
    pub fn settings_pubkey_with_bump() -> (Pubkey, u8) {
        Pubkey::find_program_address(&[SETTINGS_SEED.as_bytes()], &id())
    }

    pub fn token_pubkey_with_bump() -> (Pubkey, u8) {
        Pubkey::find_program_address(&[TOKEN_SEED.as_bytes()], &id())
    }

    pub fn lamports_account_pubkey() -> (Pubkey, u8) {
        Pubkey::find_program_address(&[LAMPORTS_SEED.as_bytes()], &id())
    }
}
