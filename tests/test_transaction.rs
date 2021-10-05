use helpers::{
    client::TestClient,
    common::{generate_mint, get_admin, get_context},
    market::TestMarket,
};
use solana_program_test::{tokio, ProgramTestContext};
use solana_sdk::{
    signature::Keypair, signer::Signer, transaction::Transaction, transport::TransportError,
};
use tokenmarket::instruction::{MarketInstructions, TokensNumber};

mod helpers;

async fn buy_sell_common(
    ctx: &mut ProgramTestContext,
    client: &TestClient,
    instruction: MarketInstructions,
) -> Result<(), TransportError> {
    let ix = match instruction {
        MarketInstructions::Buy(args) => {
            MarketInstructions::buy(&client.client.pubkey(), &client.client_token.pubkey(), args)
        }
        MarketInstructions::Sell(args) => {
            MarketInstructions::sell(&client.client.pubkey(), &client.client_token.pubkey(), args)
        }
        _ => panic!("This instruction is covered in another test"),
    };

    let blockhash = ctx.banks_client.get_recent_blockhash().await.unwrap();
    ctx.banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[ix],
            Some(&ctx.payer.pubkey()),
            &[&ctx.payer, &client.client],
            blockhash,
        ))
        .await
}

async fn buy_tokens(
    ctx: &mut ProgramTestContext,
    tokens: u64,
    client: &TestClient,
) -> Result<(), TransportError> {
    let ix = MarketInstructions::Buy(TokensNumber(tokens));
    buy_sell_common(ctx, client, ix).await
}

async fn sell_tokens(
    ctx: &mut ProgramTestContext,
    tokens: u64,
    client: &TestClient,
) -> Result<(), TransportError> {
    let ix = MarketInstructions::Sell(TokensNumber(tokens));
    buy_sell_common(ctx, client, ix).await
}

#[tokio::test]
async fn basic_transactions() {
    let ctx = &mut get_context().await;
    let mint = generate_mint(ctx).await;
    let admin = get_admin(ctx, 1_000_000_000).await;

    let mut client_lamports = 1000;
    let mut client_tokens = 15;
    let mut market_lamports = 0;
    let mut market_tokens = 10;
    let sell_price = 50;
    let buy_price = 100;

    let client = TestClient::new(ctx, &mint.pubkey(), 1000, client_tokens).await;
    let market = TestMarket::new(
        ctx,
        &admin,
        &mint.pubkey(),
        market_tokens,
        sell_price,
        buy_price,
    )
    .await;

    ////////////////////////////
    ////////////////////////////

    let tokens = 10;
    let lamports = tokens * buy_price;
    buy_tokens(ctx, tokens, &client).await.unwrap();

    market_tokens -= tokens;
    client_tokens += tokens;

    assert_eq!(client.get_balance(ctx).await, client_lamports - lamports);
    assert_eq!(market.get_balance(ctx).await, lamports);
    assert_eq!(client.get_tokens_number(ctx).await, client_tokens);
    assert_eq!(market.get_tokens_number(ctx).await, market_tokens);

    market_lamports += lamports;
    client_lamports -= lamports;

    ////////////////////////////
    ////////////////////////////

    let tokens = 10;
    let lamports = tokens * sell_price;
    sell_tokens(ctx, tokens, &client).await.unwrap();

    market_tokens += tokens;
    client_tokens -= tokens;

    assert_eq!(client.get_balance(ctx).await, client_lamports + lamports);
    assert_eq!(market.get_balance(ctx).await, market_lamports - lamports);
    assert_eq!(client.get_tokens_number(ctx).await, client_tokens);
    assert_eq!(market.get_tokens_number(ctx).await, market_tokens);

    market_lamports -= lamports;
    client_lamports += lamports;

    ////////////////////////////
    ////////////////////////////

    // InsufficientFunds
    let tokens = client_lamports / buy_price + 1;
    assert!(buy_tokens(ctx, tokens, &client).await.is_err());

    ////////////////////////////
    ////////////////////////////

    // InsufficientFunds
    let tokens = market_lamports / sell_price + 1;
    assert!(sell_tokens(ctx, tokens, &client).await.is_err());

    ////////////////////////////
    ////////////////////////////

    // Transfer 0 tokens is an error
    assert!(sell_tokens(ctx, 0, &client).await.is_err());

    ////////////////////////////
    ////////////////////////////

    let lamports = 100000;
    market.add_lamports(ctx, lamports).await;
    client.add_lamports(ctx, lamports).await;
    market_lamports += lamports;
    client_lamports += lamports;

    ////////////////////////////
    ////////////////////////////

    // InsufficientTokens
    let tokens = client_tokens + 1;
    assert!(sell_tokens(ctx, tokens, &client).await.is_err());

    ////////////////////////////
    ////////////////////////////

    // InsufficientTokens
    let tokens = market_tokens + 1;
    assert!(buy_tokens(ctx, tokens, &client).await.is_err());

    ////////////////////////////
    ////////////////////////////

    assert_eq!(client.get_balance(ctx).await, client_lamports);
    assert_eq!(market.get_balance(ctx).await, market_lamports);
    assert_eq!(client.get_tokens_number(ctx).await, client_tokens);
    assert_eq!(market.get_tokens_number(ctx).await, market_tokens);
}

#[tokio::test]
async fn complex_transactions() {
    let ctx = &mut get_context().await;
    let admin = get_admin(ctx, 1_000_000_000).await;
    let my_mint = generate_mint(ctx).await;
    let another_mint = generate_mint(ctx).await;

    let sell_price = 1;
    let buy_price = 1;
    let tokens = 100000;
    let lamports = 100000;

    let market = TestMarket::new(
        ctx,
        &admin,
        &my_mint.pubkey(),
        tokens,
        sell_price,
        buy_price,
    )
    .await;
    market.add_lamports(ctx, lamports).await;

    ////////////////////////////
    ////////////////////////////

    let another_client = TestClient::new(ctx, &another_mint.pubkey(), lamports, tokens).await;
    assert!(buy_tokens(ctx, 1, &another_client).await.is_err());
    assert!(sell_tokens(ctx, 1, &another_client).await.is_err());

    ////////////////////////////
    ////////////////////////////

    let mut strange_client = TestClient::new(ctx, &my_mint.pubkey(), lamports, tokens).await;
    strange_client.client = Keypair::new();
    assert!(buy_tokens(ctx, 1, &strange_client).await.is_err());
    assert!(sell_tokens(ctx, 1, &strange_client).await.is_err());

    ////////////////////////////
    ////////////////////////////

    let mut strange_client = TestClient::new(ctx, &my_mint.pubkey(), lamports, tokens).await;
    strange_client.client_token = Keypair::new();
    assert!(buy_tokens(ctx, 1, &strange_client).await.is_err());
    assert!(sell_tokens(ctx, 1, &strange_client).await.is_err());
}
