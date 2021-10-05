pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

solana_program::declare_id!("HpPcfd4wrNVpHVL2tXv37kWo2dXG3gLcazyU1xLha3bh");
