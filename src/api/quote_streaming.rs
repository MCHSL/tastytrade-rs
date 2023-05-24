use serde::Deserialize;

use crate::Result;
use crate::TastyTrade;

impl TastyTrade {
    pub async fn quote_streamer_tokens(&self) -> Result<QuoteStreamerTokens> {
        self.get("/quote-streamer-tokens").await
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct QuoteStreamerTokens {
    pub token: String,
    pub streamer_url: String,
    pub websocket_url: String,
    pub level: String,
}
