use tastytrade_rs::TastyTrade;

#[tokio::main]
async fn main() {
    let tasty = TastyTrade::login("username", "password", false)
        .await
        .unwrap();

    let accounts = tasty.accounts().await.unwrap();
    for account in accounts {
        println!("Account: {}", account.number().0);
        println!(
            "Positions in: {:?}",
            account
                .positions()
                .await
                .unwrap()
                .into_iter()
                .map(|p| p.symbol.0)
                .collect::<Vec<String>>()
        )
    }
}
