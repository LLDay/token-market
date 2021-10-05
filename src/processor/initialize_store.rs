use crate::{
    error::MarketError,
    id,
    instruction::PriceArgs,
    state::{MarketSettings, SETTINGS_SEED, TOKEN_SEED},
};
use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

fn init_settings_account<'info>(
    admin_info: &AccountInfo<'info>,
    market_info: &AccountInfo<'info>,
    mint_info: &AccountInfo<'info>,
    settings: &PriceArgs,
) -> ProgramResult {
    let market_settings = MarketSettings {
        admin: *admin_info.key,
        sell_price: settings.sell_price,
        buy_price: settings.buy_price,
        mint: *mint_info.key,
    };

    let (settings_pubkey, bump_seed) = MarketSettings::settings_pubkey_with_bump();
    let space = market_settings.try_to_vec()?.len();
    let signers_seed: &[&[_]] = &[SETTINGS_SEED.as_bytes(), &[bump_seed]];

    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space);

    msg!("Create settings account");
    invoke_signed(
        &system_instruction::create_account(
            admin_info.key,
            &settings_pubkey,
            lamports,
            space as u64,
            &id(),
        ),
        &[admin_info.clone(), market_info.clone()],
        &[signers_seed],
    )?;

    market_settings.serialize(&mut *market_info.data.borrow_mut())?;

    Ok(())
}

fn init_token_account<'info>(
    admin_info: &AccountInfo<'info>,
    token_info: &AccountInfo<'info>,
    market_settings_info: &AccountInfo<'info>,
    mint_info: &AccountInfo<'info>,
    rent_info: &AccountInfo<'info>,
) -> ProgramResult {
    let (token_pubkey, bump_seed) = MarketSettings::token_pubkey_with_bump();
    let signers_seed: &[&[_]] = &[TOKEN_SEED.as_bytes(), &[bump_seed]];
    let space = spl_token::state::Account::LEN;
    let rent = Rent::from_account_info(rent_info)?;
    let lamports = rent.minimum_balance(space);

    msg!("Create tokens account");
    invoke_signed(
        &system_instruction::create_account(
            admin_info.key,
            &token_pubkey,
            lamports,
            space as u64,
            &spl_token::id(),
        ),
        &[admin_info.clone(), token_info.clone()],
        &[signers_seed],
    )?;

    msg!("Initialize tokens account");
    invoke(
        &spl_token::instruction::initialize_account(
            &spl_token::id(),
            token_info.key,
            mint_info.key,
            market_settings_info.key,
        )?,
        &[
            market_settings_info.clone(),
            mint_info.clone(),
            token_info.clone(),
            rent_info.clone(),
        ],
    )?;

    Ok(())
}

pub fn process_initialize_store(accounts: &[AccountInfo], settings: PriceArgs) -> ProgramResult {
    let account_iter = &mut accounts.iter();

    let admin_info = next_account_info(account_iter)?;
    let market_settings_info = next_account_info(account_iter)?;
    let market_tokens_info = next_account_info(account_iter)?;
    let mint_info = next_account_info(account_iter)?;
    let _token_program = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;
    let rent_info = next_account_info(account_iter)?;

    if !admin_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !market_settings_info.data_is_empty() || !market_tokens_info.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if *market_tokens_info.key != MarketSettings::token_pubkey_with_bump().0 {
        return Err(MarketError::TokenPubkeyMismatch.into());
    }

    init_token_account(
        admin_info,
        market_tokens_info,
        market_settings_info,
        mint_info,
        rent_info,
    )?;
    init_settings_account(admin_info, market_settings_info, mint_info, &settings)?;

    Ok(())
}
