use serde::Deserialize;

use crate::Result;
use crate::TastyTrade;

use super::order::Symbol;

impl TastyTrade {
    pub async fn get_equity_info(&self, symbol: impl Into<Symbol>) -> Result<EquityInstrumentInfo> {
        self.get(format!("/instruments/equities/{}", symbol.into().0))
            .await
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct EquityInstrumentInfo {
    pub symbol: Symbol,
    pub streamer_symbol: String,
}
