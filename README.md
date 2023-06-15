# tastytrade-rs

Rust library for stock market trading through tastytrade's API. Very much work in progress.

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
    // Outputs:
    // DryRunResult {
    //     order: DryRunRecord {
    //         account_number: AccountNumber(
    //             "ABC12345",
    //         ),
    //         time_in_force: GTC,
    //         order_type: Limit,
    //         size: 1,
    //         underlying_symbol: Symbol(
    //             "AAPL",
    //         ),
    //         price: 170.0,
    //         price_effect: Debit,
    //         status: Received,
    //         cancellable: true,
    //         editable: true,
    //         edited: false,
    //         legs: [
    //             OrderLeg {
    //                 instrument_type: Equity,
    //                 symbol: Symbol(
    //                     "AAPL",
    //                 ),
    //                 quantity: 1,
    //                 action: BuyToOpen,
    //             },
    //         ],
    //     },
    //     warnings: [],
    //     buying_power_effect: BuyingPowerEffect {
    //         change_in_margin_requirement: 85.0,
    //         change_in_margin_requirement_effect: Debit,
    //         change_in_buying_power: 85.001,
    //         change_in_buying_power_effect: Debit,
    //         current_buying_power: 562.5,
    //         current_buying_power_effect: Credit,
    //         impact: 85.001,
    //         effect: Debit,
    //     },
    //     fee_calculation: FeeCalculation {
    //         total_fees: 0.001,
    //         total_fees_effect: Debit,
    //     },
    // },
```
