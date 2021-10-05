use helpers::{
    common::{generate_mint, get_admin, get_context},
    market::TestMarket,
};
use solana_program_test::tokio;
use solana_sdk::signer::Signer;

mod helpers;

#[tokio::test]
async fn init_market() {
    let ctx = &mut get_context().await;
    let admin = get_admin(ctx, 100_000_000).await;
    let mint = generate_mint(ctx).await;

    let lamports = 30000;
    let tokens_number = 2000;
    let sell_price = 20;
    let buy_price = 50;

    let market = TestMarket::new(
        ctx,
        &admin,
        &mint.pubkey(),
        tokens_number,
        sell_price,
        buy_price,
    )
    .await;

    let settings = market.get_settings(ctx).await;
    let maket_tokens_number = market.get_tokens_number(ctx).await;

    assert_eq!(settings.buy_price, buy_price);
    assert_eq!(settings.sell_price, sell_price);
    assert_eq!(maket_tokens_number, tokens_number);

    assert_eq!(market.get_balance(ctx).await, 0);
    market.add_lamports(ctx, lamports).await;
    assert_eq!(market.get_balance(ctx).await, lamports);
}

#[tokio::test]
async fn update_market() {
    let ctx = &mut get_context().await;
    let mint = generate_mint(ctx).await;
    let admin = get_admin(ctx, 1_000_000_000).await;
    let market = TestMarket::new(ctx, &admin, &mint.pubkey(), 0, 1, 2).await;

    let settings = market.get_settings(ctx).await;
    assert_eq!(settings.sell_price, 1);
    assert_eq!(settings.buy_price, 2);

    market.update(ctx, &admin, 100, 200).await;

    let settings = market.get_settings(ctx).await;
    assert_eq!(settings.sell_price, 100);
    assert_eq!(settings.buy_price, 200);
}

#[tokio::test]
#[should_panic]
async fn market_initalized_twice() {
    let ctx = &mut get_context().await;
    let admin = get_admin(ctx, 100_000_000).await;
    let mint = generate_mint(ctx).await;

    let market = TestMarket {};
    market
        .initialize(ctx, &admin, &mint.pubkey(), 120, 0, 0)
        .await;

    market
        .initialize(ctx, &admin, &mint.pubkey(), 140, 0, 0)
        .await;
}
