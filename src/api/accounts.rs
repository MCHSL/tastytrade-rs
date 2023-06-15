use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::api::base::Result;
use crate::client::TastyTrade;

use super::base::Items;
use super::order::{DryRunResult, LiveOrderRecord, Order, OrderId, OrderPlacedResult};
use super::position::Position;

impl TastyTrade {
    pub async fn accounts(&self) -> Result<Vec<Account>> {
        let resp: Items<AccountInner> = self.get("/customers/me/accounts").await?;
        Ok(resp
            .items
            .into_iter()
            .map(|inner| Account { inner, tasty: self })
            .collect())
    }

    pub async fn account(
        &self,
        account_number: impl Into<AccountNumber>,
    ) -> Result<Option<Account>> {
        let account_number = account_number.into();
        let accounts = self.accounts().await?;
        for account in accounts {
            if account.inner.account.account_number == account_number {
                return Ok(Some(account));
            }
        }
        Ok(None)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(transparent)]
pub struct AccountNumber(pub String);

impl<T: AsRef<str>> From<T> for AccountNumber {
    fn from(value: T) -> Self {
        Self(value.as_ref().to_owned())
    }
}

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
    pub(crate) inner: AccountInner,
    tasty: &'t TastyTrade,
}

impl<'t> Account<'t> {
    pub fn number(&self) -> AccountNumber {
        self.inner.account.account_number.clone()
    }

    pub async fn balance(&self) -> Result<Balance> {
        let resp = self
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
                &format!(
                    "/accounts/{}/orders/dry-run",
                    self.inner.account.account_number.0
                ),
                order,
            )
            .await?;
        Ok(resp)
    }

    pub async fn place_order(&self, order: &Order) -> Result<OrderPlacedResult> {
        let resp: OrderPlacedResult = self
            .tasty
            .post(
                &format!("/accounts/{}/orders", self.inner.account.account_number.0),
                order,
            )
            .await?;
        Ok(resp)
    }

    pub async fn cancel_order(&self, id: OrderId) -> Result<LiveOrderRecord> {
        self.tasty
            .delete(&format!(
                "/accounts/{}/orders/{}",
                self.inner.account.account_number.0, id.0
            ))
            .await
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Balance {
    pub account_number: AccountNumber,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub cash_balance: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub long_equity_value: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub short_equity_value: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub long_derivative_value: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub short_derivative_value: Decimal,
}
