use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Position {
    pub account_number: String,
    pub symbol: String,
    pub instrument_type: String,
    pub underlying_symbol: String,
    pub quantity: u64,
    pub quantity_direction: String,
}
