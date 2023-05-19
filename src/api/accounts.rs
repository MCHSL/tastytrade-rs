use serde::{Deserialize, Serialize};

use crate::api::base::Result;
use crate::client::TastyTrade;

use super::base::Items;
use super::order::{DryRunResult, LiveOrderRecord, Order};
use super::position::Position;

pub async fn accounts(tasty: &TastyTrade) -> Result<Vec<Account>> {
    let resp: Items<AccountInner> = tasty.get("/customers/me/accounts").await?;
    Ok(resp
        .items
        .into_iter()
        .map(|inner| Account { inner, tasty })
        .collect())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AccountNumber(String);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AccountDetails {
    pub account_number: AccountNumber,
    pub external_id: Option<String>,
    pub opened_at: String,
    pub nickname: String,
    pub account_type_name: String,
    pub day_trader_status: bool,
    pub is_firm_error: bool,
    pub is_firm_proprietary: bool,
    pub is_test_drive: bool,
    pub margin_or_cash: String,
    pub is_foreign: bool,
    pub funding_date: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AccountInner {
    pub account: AccountDetails,
    pub authority_level: String,
}

pub struct Account<'t> {
    inner: AccountInner,
    tasty: &'t TastyTrade,
}

impl<'t> Account<'t> {
    pub async fn balance(&self) -> Result<Balance> {
        let resp: Balance = self
            .tasty
            .get(&format!(
                "/accounts/{}/balances",
                self.inner.account.account_number.0
            ))
            .await?;
        Ok(resp)
    }

    pub async fn positions(&self) -> Result<Vec<Position>> {
        let resp: Items<Position> = self
            .tasty
            .get(&format!(
                "/accounts/{}/positions",
                self.inner.account.account_number.0
            ))
            .await?;
        Ok(resp.items)
    }

    pub async fn live_orders(&self) -> Result<Vec<LiveOrderRecord>> {
        let resp: Items<LiveOrderRecord> = self
            .tasty
            .get(&format!(
                "/accounts/{}/orders/live",
                self.inner.account.account_number.0
            ))
            .await?;
        Ok(resp.items)
    }

    pub async fn dry_run(&self, order: &Order) -> Result<DryRunResult> {
        let resp: DryRunResult = self
            .tasty
            .post(
                &format!("/accounts/{}/orders", self.inner.account.account_number.0),
                order,
            )
            .await?;
        Ok(resp)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Balance {
    pub account_number: AccountNumber,
    pub cash_balance: String,
    pub long_equity_value: String,
    pub short_equity_value: String,
    pub long_derivative_value: String,
    pub short_derivative_value: String,
}
