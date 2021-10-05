use crate::{
    error::MarketError,
    instruction::TokensNumber,
    state::{MarketSettings, LAMPORTS_SEED, SETTINGS_SEED},
};
use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
};

pub enum OperationType {
    Buy,
    Sell,
}

fn process_buy<'info>(
    tokens_number: u64,
    buy_price: u64,
    market_settings_info: &AccountInfo<'info>,
    market_token_info: &AccountInfo<'info>,
    market_lamports_info: &AccountInfo<'info>,
    client_info: &AccountInfo<'info>,
    client_token_info: &AccountInfo<'info>,
) -> ProgramResult {
    let lamports = buy_price
        .checked_mul(tokens_number)
        .ok_or(MarketError::TooManyLamports)?;

    if client_info.lamports() < lamports {
        return Err(ProgramError::InsufficientFunds);
    }

    let ix = solana_program::system_instruction::transfer(
        client_info.key,
        market_lamports_info.key,
        lamports,
    );

    msg!("Transfer {} lamports to the market", lamports);
    invoke(&ix, &[client_info.clone(), market_lamports_info.clone()])?;

    let ix = spl_token::instruction::transfer(
        &spl_token::id(),
        market_token_info.key,
        client_token_info.key,
        market_settings_info.key,
        &[market_settings_info.key],
        tokens_number,
    )?;

    let bump = MarketSettings::settings_pubkey_with_bump().1;
    let seed: &[&[_]] = &[SETTINGS_SEED.as_bytes(), &[bump]];

    msg!("Transfer {} tokens to the client", tokens_number);
    invoke_signed(
        &ix,
        &[
            market_settings_info.clone(),
            market_token_info.clone(),
            client_token_info.clone(),
        ],
        &[seed],
    )?;

    Ok(())
}

fn process_sell<'info>(
    tokens_number: u64,
    sell_price: u64,
    market_token_info: &AccountInfo<'info>,
    market_lamports_info: &AccountInfo<'info>,
    client_info: &AccountInfo<'info>,
    client_token_info: &AccountInfo<'info>,
) -> ProgramResult {
    let lamports = sell_price
        .checked_mul(tokens_number)
        .ok_or(MarketError::TooManyLamports)?;

    if market_lamports_info.lamports() < lamports {
        return Err(ProgramError::InsufficientFunds);
    }

    let bump = MarketSettings::lamports_account_pubkey().1;
    let seed: &[&[_]] = &[LAMPORTS_SEED.as_bytes(), &[bump]];
    let ix = solana_program::system_instruction::transfer(
        market_lamports_info.key,
        client_info.key,
        lamports,
    );

    msg!("Transfer {} lamports to the client", lamports);
    invoke_signed(
        &ix,
        &[client_info.clone(), market_lamports_info.clone()],
        &[seed],
    )?;

    let ix = spl_token::instruction::transfer(
        &spl_token::id(),
        client_token_info.key,
        market_token_info.key,
        client_info.key,
        &[client_info.key],
        tokens_number,
    )?;

    msg!("Transfer {} tokens to the market", tokens_number);
    invoke(
        &ix,
        &[
            market_token_info.clone(),
            client_token_info.clone(),
            client_info.clone(),
        ],
    )?;

    Ok(())
}

pub fn process_buy_sell(
    accounts: &[AccountInfo],
    tokens_number: TokensNumber,
    operation: OperationType,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();

    let client_info = next_account_info(account_iter)?;
    let client_token_info = next_account_info(account_iter)?;
    let market_lamports_info = next_account_info(account_iter)?;
    let market_settings_info = next_account_info(account_iter)?;
    let market_token_info = next_account_info(account_iter)?;

    let market_settings = MarketSettings::try_from_slice(&market_settings_info.data.borrow())?;
    let market_token_account = spl_token::state::Account::unpack(&market_token_info.data.borrow())?;
    let client_token_account = spl_token::state::Account::unpack(&client_token_info.data.borrow())?;
    let expected_settings_pubkey = MarketSettings::settings_pubkey_with_bump().0;
    let expected_token_pubkey = MarketSettings::token_pubkey_with_bump().0;
    let tokens_number = tokens_number.0;

    if !client_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if tokens_number == 0 {
        return Err(ProgramError::InvalidArgument);
    }

    if client_token_account.owner != *client_info.key {
        return Err(ProgramError::InvalidArgument);
    }

    if client_info.key == market_settings_info.key {
        return Err(MarketError::SelfTransaction.into());
    }

    if client_token_account.mint != market_token_account.mint {
        return Err(MarketError::UnsupportedMint.into());
    }

    if expected_settings_pubkey != *market_settings_info.key {
        return Err(MarketError::SettingsPubkeyMismatch.into());
    }

    if expected_token_pubkey != *market_token_info.key {
        return Err(MarketError::TokenPubkeyMismatch.into());
    }

    match operation {
        OperationType::Buy => process_buy(
            tokens_number,
            market_settings.buy_price,
            market_settings_info,
            market_token_info,
            market_lamports_info,
            client_info,
            client_token_info,
        ),
        OperationType::Sell => process_sell(
            tokens_number,
            market_settings.sell_price,
            market_token_info,
            market_lamports_info,
            client_info,
            client_token_info,
        ),
    }
}
