use crate::{error::MarketError, instruction::PriceArgs, state::MarketSettings};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
};

pub fn process_update_price(accounts: &[AccountInfo], settings: PriceArgs) -> ProgramResult {
    let account_iter = &mut accounts.iter();

    let admin_info = next_account_info(account_iter)?;
    let market_info = next_account_info(account_iter)?;

    if !admin_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let expected_settings_pubkey = MarketSettings::settings_pubkey_with_bump().0;
    if *market_info.key != expected_settings_pubkey {
        return Err(MarketError::SettingsPubkeyMismatch.into());
    }

    let mut market_settings = MarketSettings::try_from_slice(&market_info.data.borrow())?;
    if market_settings.admin != *admin_info.key {
        return Err(ProgramError::IllegalOwner);
    }

    market_settings.sell_price = settings.sell_price;
    market_settings.buy_price = settings.buy_price;

    msg!("Updating price");
    market_settings.serialize(&mut *market_info.data.borrow_mut())?;

    Ok(())
}
