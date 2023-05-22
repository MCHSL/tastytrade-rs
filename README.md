# tastytrade-rs

Rust library for stock market trading through tastytrade's API. Very incomplete and only tested on the demo environment.

# Example

```rust
    let tasty = TastyTrade::login("username", "password", false)
        .await
        .unwrap();

    let account = tasty.account("ABC12345")
        .await
        .unwrap()
        .unwrap();
    println!("{:#?}", account.balance().await);
    println!("{:#?}", account.positions().await);
    println!("{:#?}", account.live_orders().await);

    let order_leg = OrderLegBuilder::default()
        .instrument_type(InstrumentType::Equity)
        .symbol("AAPL")
        .quantity(1u64)
        .action(Action::BuyToOpen)
        .build()
        .unwrap();

    let order = OrderBuilder::default()
        .time_in_force(TimeInForce::GTC)
        .order_type(OrderType::Limit)
        .price(dec!(170.0))
        .price_effect(PriceEffect::Debit)
        .legs(vec![order_leg])
        .build()
        .unwrap();

    let dry_result = account.dry_run(&order).await;

    println!("{dry_result:#?}");
```
