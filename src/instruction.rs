use crate::{id, state::MarketSettings};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct PriceArgs {
    pub sell_price: u64,
    pub buy_price: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct TokensNumber(pub u64);

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub enum MarketInstructions {
    /// Initialize store
    ///
    /// 0. `[signer, writable]` Market's admin
    /// 1. `[writable]` Market's account with settings, PDA
    /// 2. `[writable]` Market's token account, PDA
    /// 3. `[]` Mint account
    /// 4. `[]` Token program
    /// 5. `[]` System program
    /// 6. `[]` Rent sysvar
    InitializeStore(PriceArgs),

    /// Update price
    ///
    /// 0. `[signer]` Market's admin
    /// 1. `[writable]` Market's account with settings, PDA
    UpdatePrice(PriceArgs),

    /// Sell
    ///
    /// 0. `[signer, writable]` Client's account
    /// 1. `[writable]` Client's token account
    /// 2. `[writable]` Market's account with lamports, PDA
    /// 3. `[]` Market's account with settings, PDA
    /// 4. `[writable]` Market's token account, PDA
    /// 5. `[]` Token program
    /// 6. `[]` System program
    Sell(TokensNumber),

    /// Buy
    ///
    /// 0. `[signer, writable]` Client's account
    /// 1. `[writable]` Client's token account
    /// 2. `[writable]` Market's account with lamports, PDA
    /// 3. `[writable]` Market's account with settings, PDA
    /// 4. `[writable]` Market's token account, PDA
    /// 5. `[]` Token program
    /// 6. `[]` System program
    Buy(TokensNumber),
}

impl MarketInstructions {
    pub fn initialize_store(admin: &Pubkey, mint: &Pubkey, args: PriceArgs) -> Instruction {
        let market_pubkey = MarketSettings::settings_pubkey_with_bump().0;
        let market_token_pubkey = MarketSettings::token_pubkey_with_bump().0;
        let token_program = spl_token::id();
        let system_program = system_program::id();
        let rent_sysvar = sysvar::rent::id();

        Instruction::new_with_borsh(
            id(),
            &MarketInstructions::InitializeStore(args),
            vec![
                AccountMeta::new(*admin, true),
                AccountMeta::new(market_pubkey, false),
                AccountMeta::new(market_token_pubkey, false),
                AccountMeta::new_readonly(*mint, false),
                AccountMeta::new_readonly(token_program, false),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(rent_sysvar, false),
            ],
        )
    }

    pub fn update_price(admin: &Pubkey, args: PriceArgs) -> Instruction {
        let market_pubkey = MarketSettings::settings_pubkey_with_bump().0;

        Instruction::new_with_borsh(
            id(),
            &MarketInstructions::UpdatePrice(args),
            vec![
                AccountMeta::new_readonly(*admin, true),
                AccountMeta::new(market_pubkey, false),
            ],
        )
    }

    fn sell_buy_common(
        client: &Pubkey,
        client_token: &Pubkey,
        data: &MarketInstructions,
    ) -> Instruction {
        let market_settings_pubkey = MarketSettings::settings_pubkey_with_bump().0;
        let market_token_pubkey = MarketSettings::token_pubkey_with_bump().0;
        let market_lamports_pubkey = MarketSettings::lamports_account_pubkey().0;
        let token_program = spl_token::id();
        let system_program = system_program::id();

        Instruction::new_with_borsh(
            id(),
            data,
            vec![
                AccountMeta::new(*client, true),
                AccountMeta::new(*client_token, false),
                AccountMeta::new(market_lamports_pubkey, false),
                AccountMeta::new_readonly(market_settings_pubkey, false),
                AccountMeta::new(market_token_pubkey, false),
                AccountMeta::new_readonly(token_program, false),
                AccountMeta::new_readonly(system_program, false),
            ],
        )
    }

    pub fn sell(client: &Pubkey, client_token: &Pubkey, args: TokensNumber) -> Instruction {
        let data = MarketInstructions::Sell(args);
        MarketInstructions::sell_buy_common(client, client_token, &data)
    }

    pub fn buy(client: &Pubkey, client_token: &Pubkey, args: TokensNumber) -> Instruction {
        let data = MarketInstructions::Buy(args);
        MarketInstructions::sell_buy_common(client, client_token, &data)
    }
}
