use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct LiveOrderRecord {
    pub id: u64,
    pub account_number: String,
    pub time_in_force: String,
    pub order_type: String,
    pub size: u64,
    pub underlying_symbol: String,
    pub price: String,
    pub price_effect: String,
    pub status: String,
    pub cancellable: bool,
    pub editable: bool,
    pub edited: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct LiveOrderLeg {
    pub instrument_type: String,
    pub symbol: String,
    pub quantity: u64,
    pub remaining_quantity: u64,
    pub action: String,
    pub fills: Vec<String>,
}

#[derive(Builder, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Order {
    time_in_force: String,
    order_type: String,
    price: String,
    price_effect: String,
    legs: Vec<OrderLeg>,
}

#[derive(Builder, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct OrderLeg {
    instrument_type: String,
    symbol: String,
    quantity: u64,
    action: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DryRunResult {
    pub order: LiveOrderRecord,
    pub warnings: Vec<Warning>,
    pub buying_power_effect: BuyingPowerEffect,
    pub fee_calculation: FeeCalculation,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BuyingPowerEffect {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FeeCalculation {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Warning {}
