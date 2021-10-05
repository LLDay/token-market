use crate::instruction::MarketInstructions;
use borsh::BorshDeserialize;
use buy_sell::{process_buy_sell, OperationType};
use initialize_store::process_initialize_store;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};
use update_price::process_update_price;

pub mod buy_sell;
pub mod initialize_store;
pub mod update_price;

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = MarketInstructions::try_from_slice(instruction_data)?;
    match instruction {
        MarketInstructions::InitializeStore(args) => process_initialize_store(accounts, args),
        MarketInstructions::UpdatePrice(args) => process_update_price(accounts, args),
        MarketInstructions::Buy(args) => process_buy_sell(accounts, args, OperationType::Buy),
        MarketInstructions::Sell(args) => process_buy_sell(accounts, args, OperationType::Sell),
    }
}
