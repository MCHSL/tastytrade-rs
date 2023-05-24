use std::collections::HashMap;

use crate::api::base::Result;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::Value;

use crate::TastyTrade;

use super::{base::Items, order::Symbol};

pub async fn nested_option_chain_for(
    tasty: &TastyTrade,
    symbol: impl Into<Symbol>,
) -> Result<NestedOptionChain> {
    let mut resp: Items<NestedOptionChain> = tasty
        .get(format!("/option-chains/{}/nested", symbol.into().0))
        .await?;
    Ok(resp.items.remove(0))
}

pub async fn option_chain_for(
    tasty: &TastyTrade,
    symbol: impl Into<Symbol>,
) -> Result<Vec<OptionChain>> {
    let resp: Items<OptionChain> = tasty
        .get(format!("/option-chains/{}", symbol.into().0))
        .await?;
    Ok(resp.items)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct NestedOptionChain {
    pub underlying_symbol: Symbol,
    pub root_symbol: Symbol,
    pub option_chain_type: String,
    pub shares_per_contract: u64,
    pub expirations: Vec<Expiration>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Expiration {
    pub expiration_type: String,
    pub expiration_date: String,
    pub days_to_expiration: u64,
    pub settlement_type: String,
    pub strikes: Vec<Strike>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Strike {
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub strike_price: Decimal,
    pub call: Symbol,
    pub put: Symbol,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct OptionChain {
    pub underlying_symbol: Symbol,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub strike_price: Decimal,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}
