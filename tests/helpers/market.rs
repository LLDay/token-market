use solana_program::{pubkey::Pubkey, system_instruction};
use solana_program_test::ProgramTestContext;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};
use tokenmarket::{
    instruction::{MarketInstructions, PriceArgs},
    state::MarketSettings,
};

pub struct TestMarket {}

impl TestMarket {
    pub async fn new(
        ctx: &mut ProgramTestContext,
        admin: &Keypair,
        mint: &Pubkey,
        tokens: u64,
        sell_price: u64,
        buy_price: u64,
    ) -> TestMarket {
        let market = TestMarket {};
        market
            .initialize(ctx, admin, mint, tokens, sell_price, buy_price)
            .await;
        market
    }

    pub async fn add_lamports(&self, ctx: &mut ProgramTestContext, lamports: u64) {
        let pubkey = MarketSettings::lamports_account_pubkey().0;
        let ix = [system_instruction::transfer(
            &ctx.payer.pubkey(),
            &pubkey,
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

    pub async fn initialize(
        &self,
        ctx: &mut ProgramTestContext,
        admin: &Keypair,
        mint: &Pubkey,
        tokens: u64,
        sell_price: u64,
        buy_price: u64,
    ) {
        let args = PriceArgs {
            sell_price,
            buy_price,
        };

        let market_token_pubkey = MarketSettings::token_pubkey_with_bump().0;

        let ixs = [
            MarketInstructions::initialize_store(&admin.pubkey(), mint, args),
            spl_token::instruction::mint_to(
                &spl_token::id(),
                mint,
                &market_token_pubkey,
                &ctx.payer.pubkey(),
                &[&ctx.payer.pubkey()],
                tokens,
            )
            .unwrap(),
        ];
        ctx.banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &ixs,
                Some(&ctx.payer.pubkey()),
                &[&ctx.payer, admin],
                ctx.last_blockhash,
            ))
            .await
            .unwrap();
    }

    pub async fn update(
        &self,
        ctx: &mut ProgramTestContext,
        admin: &Keypair,
        sell_price: u64,
        buy_price: u64,
    ) {
        let args = PriceArgs {
            sell_price,
            buy_price,
        };
        let ix = MarketInstructions::update_price(&admin.pubkey(), args);
        ctx.banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &[ix],
                Some(&admin.pubkey()),
                &[admin],
                ctx.last_blockhash,
            ))
            .await
            .unwrap();
    }

    pub async fn get_settings(&self, ctx: &mut ProgramTestContext) -> MarketSettings {
        let settings_pubkey = MarketSettings::settings_pubkey_with_bump().0;
        ctx.banks_client
            .get_account_data_with_borsh(settings_pubkey)
            .await
            .unwrap()
    }

    pub async fn get_tokens_number(&self, ctx: &mut ProgramTestContext) -> u64 {
        let token_pubkey = MarketSettings::token_pubkey_with_bump().0;
        let token_account = ctx
            .banks_client
            .get_packed_account_data::<spl_token::state::Account>(token_pubkey)
            .await
            .unwrap();
        token_account.amount
    }

    pub async fn get_balance(&self, ctx: &mut ProgramTestContext) -> u64 {
        let lamports_pubkey = MarketSettings::lamports_account_pubkey().0;
        ctx.banks_client.get_balance(lamports_pubkey).await.unwrap()
    }
}
