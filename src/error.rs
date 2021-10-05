use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Clone, Debug, Error, FromPrimitive)]
pub enum MarketError {
    #[error("The pubkey of the token is different from PDA")]
    TokenPubkeyMismatch,

    #[error("The pubkey of the market's settings is different from PDA")]
    SettingsPubkeyMismatch,

    #[error("Market cannot buy from or sell to itself")]
    SelfTransaction,

    #[error("The limit of possible lamports has been exceeded")]
    TooManyLamports,

    #[error("The market doesn't support the mint")]
    UnsupportedMint,
}

impl From<MarketError> for ProgramError {
    fn from(error: MarketError) -> Self {
        ProgramError::Custom(error as u32)
    }
}

impl PrintProgramError for MarketError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl<T> DecodeError<T> for MarketError {
    fn type_of() -> &'static str {
        "Store Error"
    }
}
