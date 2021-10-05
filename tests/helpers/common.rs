use solana_program::{program_pack::Pack, pubkey::Pubkey, rent::Rent, system_instruction};
use solana_program_test::{processor, ProgramTest, ProgramTestContext};
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};
use spl_token::state::Mint;
use tokenmarket::{entrypoint, id};

pub async fn get_context() -> ProgramTestContext {
    let program_test = ProgramTest::new(
        "tokenmarket",
        id(),
        processor!(entrypoint::process_instruction),
    );
    program_test.start_with_context().await
}

pub async fn get_admin(ctx: &mut ProgramTestContext, lamports: u64) -> Keypair {
    let admin = Keypair::new();
    let ix = system_instruction::transfer(&ctx.payer.pubkey(), &admin.pubkey(), lamports);
    ctx.banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[ix],
            Some(&ctx.payer.pubkey()),
            &[&ctx.payer],
            ctx.last_blockhash,
        ))
        .await
        .unwrap();
    admin
}

pub async fn generate_mint(ctx: &mut ProgramTestContext) -> Keypair {
    let mint = Keypair::new();
    let decimals = 0;

    let ixs = [
        system_instruction::create_account(
            &ctx.payer.pubkey(),
            &mint.pubkey(),
            Rent::default().minimum_balance(Mint::LEN),
            Mint::LEN.try_into().unwrap(),
            &spl_token::id(),
        ),
        spl_token::instruction::initialize_mint(
            &spl_token::id(),
            &mint.pubkey(),
            &ctx.payer.pubkey(),
            None,
            decimals,
        )
        .unwrap(),
    ];

    ctx.banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &ixs,
            Some(&ctx.payer.pubkey()),
            &[&ctx.payer, &mint],
            ctx.last_blockhash,
        ))
        .await
        .unwrap();

    mint
}
