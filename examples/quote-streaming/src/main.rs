use tastytrade_rs::dxfeed;
use tastytrade_rs::TastyTrade;

use dxfeed::EventData::Quote;

#[tokio::main]
async fn main() {
    let tasty = TastyTrade::login("username", "password", false)
        .await
        .unwrap();

    let mut streamer = tasty.create_quote_streamer().await.unwrap();
    streamer.subscribe(&["SPX"]);

    streamer
        .handle_events(|ev| {
            if let Quote(data) = ev.data {
                println!("{}: {}/{}", ev.sym, data.bid_price, data.ask_price);
            }
        })
        .await
}
