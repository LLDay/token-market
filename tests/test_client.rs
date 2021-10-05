use helpers::{
    client::TestClient,
    common::{generate_mint, get_context},
};
use solana_program_test::tokio;
use solana_sdk::signer::Signer;

mod helpers;

#[tokio::test]
async fn test_client() {
    let tokens = 30;
    let lamports = 1000;

    let ctx = &mut get_context().await;
    let mint = generate_mint(ctx).await;
    let client = TestClient::new(ctx, &mint.pubkey(), lamports, tokens).await;

    assert_eq!(client.get_tokens_number(ctx).await, tokens);
    assert_eq!(client.get_balance(ctx).await, lamports);

    client.add_lamports(ctx, lamports).await;
    assert_eq!(client.get_balance(ctx).await, lamports * 2);
}
