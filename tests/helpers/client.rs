use solana_program::{program_pack::Pack, pubkey::Pubkey, rent::Rent, system_instruction};
use solana_program_test::ProgramTestContext;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};
use spl_token::state::Account;

pub struct TestClient {
    pub client: Keypair,
    pub client_token: Keypair,
}

impl TestClient {
    pub async fn new(
        ctx: &mut ProgramTestContext,
        mint: &Pubkey,
        lamports: u64,
        tokens: u64,
    ) -> TestClient {
        let client = Keypair::new();
        let client_token = Keypair::new();

        let ixs = [
            system_instruction::transfer(&ctx.payer.pubkey(), &client.pubkey(), lamports),
            system_instruction::create_account(
                &ctx.payer.pubkey(),
                &client_token.pubkey(),
                Rent::default().minimum_balance(Account::LEN),
                Account::LEN.try_into().unwrap(),
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &client_token.pubkey(),
                mint,
                &client.pubkey(),
            )
            .unwrap(),
            spl_token::instruction::mint_to(
                &spl_token::id(),
                mint,
                &client_token.pubkey(),
                &ctx.payer.pubkey(),
                &[&client.pubkey()],
                tokens,
            )
            .unwrap(),
        ];

        ctx.banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &ixs,
                Some(&ctx.payer.pubkey()),
                &[&ctx.payer, &client_token, &client],
                ctx.last_blockhash,
            ))
            .await
            .unwrap();

        TestClient {
            client,
            client_token,
        }
    }

    pub async fn add_lamports(&self, ctx: &mut ProgramTestContext, lamports: u64) {
        let ix = [system_instruction::transfer(
            &ctx.payer.pubkey(),
            &self.client.pubkey(),
            lamports,
        )];

        ctx.banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &ix,
                Some(&ctx.payer.pubkey()),
                &[&ctx.payer],
                ctx.last_blockhash,
            ))
            .await
            .unwrap();
    }

    pub async fn get_tokens_number(&self, ctx: &mut ProgramTestContext) -> u64 {
        let token_account = ctx
            .banks_client
            .get_packed_account_data::<spl_token::state::Account>(self.client_token.pubkey())
            .await
            .unwrap();
        token_account.amount
    }

    pub async fn get_balance(&self, ctx: &mut ProgramTestContext) -> u64 {
        ctx.banks_client
            .get_balance(self.client.pubkey())
            .await
            .unwrap()
    }
}
