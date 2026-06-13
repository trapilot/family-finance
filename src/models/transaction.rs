use rusqlite::Row;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use super::wallet::{Currency, to_sqlite_err};
use super::{decimal_to_f64, f64_to_decimal};
use crate::error::AppError;

// ─── TransactionType ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    Income,
    Expense,
    Transfer,
    InvestBuy,
    InvestSell,
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionType::Income    => write!(f, "income"),
            TransactionType::Expense   => write!(f, "expense"),
            TransactionType::Transfer  => write!(f, "transfer"),
            TransactionType::InvestBuy => write!(f, "invest_buy"),
            TransactionType::InvestSell => write!(f, "invest_sell"),
        }
    }
}

impl FromStr for TransactionType {
    type Err = AppError;
    fn from_str(s: &str) -> crate::error::Result<Self> {
        match s {
            "income"      => Ok(TransactionType::Income),
            "expense"     => Ok(TransactionType::Expense),
            "transfer"    => Ok(TransactionType::Transfer),
            "invest_buy"  => Ok(TransactionType::InvestBuy),
            "invest_sell" => Ok(TransactionType::InvestSell),
            _             => Err(AppError::Parse(format!("Unknown transaction type: {s}"))),
        }
    }
}

// ─── IncomeType ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncomeType {
    Salary,
    Bonus,
    SideIncome,
    Rental,
    Investment,
    Other,
}

impl fmt::Display for IncomeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IncomeType::Salary     => write!(f, "salary"),
            IncomeType::Bonus      => write!(f, "bonus"),
            IncomeType::SideIncome => write!(f, "side_income"),
            IncomeType::Rental     => write!(f, "rental"),
            IncomeType::Investment => write!(f, "investment"),
            IncomeType::Other      => write!(f, "other"),
        }
    }
}

impl FromStr for IncomeType {
    type Err = AppError;
    fn from_str(s: &str) -> crate::error::Result<Self> {
        match s {
            "salary"      => Ok(IncomeType::Salary),
            "bonus"       => Ok(IncomeType::Bonus),
            "side_income" => Ok(IncomeType::SideIncome),
            "rental"      => Ok(IncomeType::Rental),
            "investment"  => Ok(IncomeType::Investment),
            "other"       => Ok(IncomeType::Other),
            _             => Err(AppError::Parse(format!("Unknown income type: {s}"))),
        }
    }
}

// ─── Transaction ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub id:             String,
    pub txn_type:       TransactionType,
    pub wallet_id:      String,
    pub amount:         Decimal,
    pub currency:       Currency,

    // income
    pub income_type:    Option<IncomeType>,

    // expense
    pub category_id:    Option<String>,

    // transfer
    pub to_wallet_id:   Option<String>,
    pub to_amount:      Option<Decimal>,
    pub to_currency:    Option<Currency>,

    // invest_buy / invest_sell
    pub holding_id:     Option<String>,
    pub asset_quantity: Option<Decimal>,
    pub asset_price:    Option<Decimal>,

    pub note:       Option<String>,
    pub txn_date:   i64,
    pub created_at: i64,
}

impl Transaction {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let txn_type_str: String = row.get("txn_type")?;
        let currency_str: String = row.get("currency")?;

        let to_currency: Option<String> = row.get("to_currency")?;
        let income_type_str: Option<String> = row.get("income_type")?;

        let amount_f64: f64 = row.get("amount")?;
        let to_amount_f64: Option<f64> = row.get("to_amount")?;
        let asset_qty_f64: Option<f64> = row.get("asset_quantity")?;
        let asset_price_f64: Option<f64> = row.get("asset_price")?;

        Ok(Transaction {
            id:             row.get("id")?,
            txn_type:       TransactionType::from_str(&txn_type_str).map_err(to_sqlite_err)?,
            wallet_id:      row.get("wallet_id")?,
            amount:         f64_to_decimal(amount_f64),
            currency:       Currency::from_str(&currency_str).map_err(to_sqlite_err)?,
            income_type:    income_type_str
                                .as_deref()
                                .map(IncomeType::from_str)
                                .transpose()
                                .map_err(to_sqlite_err)?,
            category_id:    row.get("category_id")?,
            to_wallet_id:   row.get("to_wallet_id")?,
            to_amount:      to_amount_f64.map(f64_to_decimal),
            to_currency:    to_currency
                                .as_deref()
                                .map(Currency::from_str)
                                .transpose()
                                .map_err(to_sqlite_err)?,
            holding_id:     row.get("holding_id")?,
            asset_quantity: asset_qty_f64.map(f64_to_decimal),
            asset_price:    asset_price_f64.map(f64_to_decimal),
            note:           row.get("note")?,
            txn_date:       row.get("txn_date")?,
            created_at:     row.get("created_at")?,
        })
    }

    pub fn amount_f64(&self) -> f64 { decimal_to_f64(self.amount) }
}

// ─── NewTransaction (input DTO) ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct NewTransaction {
    pub txn_type:       TransactionType,
    pub wallet_id:      String,
    pub amount:         Decimal,
    pub currency:       Currency,

    pub income_type:    Option<IncomeType>,
    pub category_id:    Option<String>,

    pub to_wallet_id:   Option<String>,
    pub to_amount:      Option<Decimal>,
    pub to_currency:    Option<Currency>,

    pub holding_id:     Option<String>,
    pub asset_quantity: Option<Decimal>,
    pub asset_price:    Option<Decimal>,

    pub note:     Option<String>,
    pub txn_date: i64,
}

impl NewTransaction {
    pub fn income(wallet_id: &str, amount: Decimal, currency: Currency, income_type: IncomeType, txn_date: i64) -> Self {
        Self {
            txn_type: TransactionType::Income,
            wallet_id: wallet_id.to_string(),
            amount,
            currency,
            income_type: Some(income_type),
            category_id: None,
            to_wallet_id: None,
            to_amount: None,
            to_currency: None,
            holding_id: None,
            asset_quantity: None,
            asset_price: None,
            note: None,
            txn_date,
        }
    }

    pub fn expense(wallet_id: &str, amount: Decimal, currency: Currency, category_id: &str, txn_date: i64) -> Self {
        Self {
            txn_type: TransactionType::Expense,
            wallet_id: wallet_id.to_string(),
            amount,
            currency,
            income_type: None,
            category_id: Some(category_id.to_string()),
            to_wallet_id: None,
            to_amount: None,
            to_currency: None,
            holding_id: None,
            asset_quantity: None,
            asset_price: None,
            note: None,
            txn_date,
        }
    }

    pub fn transfer(
        from_wallet_id: &str,
        amount: Decimal,
        currency: Currency,
        to_wallet_id: &str,
        to_amount: Decimal,
        to_currency: Currency,
        txn_date: i64,
    ) -> Self {
        Self {
            txn_type: TransactionType::Transfer,
            wallet_id: from_wallet_id.to_string(),
            amount,
            currency,
            income_type: None,
            category_id: None,
            to_wallet_id: Some(to_wallet_id.to_string()),
            to_amount: Some(to_amount),
            to_currency: Some(to_currency),
            holding_id: None,
            asset_quantity: None,
            asset_price: None,
            note: None,
            txn_date,
        }
    }

    pub fn invest_buy(wallet_id: &str, holding_id: &str, quantity: Decimal, price: Decimal, currency: Currency, txn_date: i64) -> Self {
        let total = quantity * price;
        Self {
            txn_type: TransactionType::InvestBuy,
            wallet_id: wallet_id.to_string(),
            amount: total,
            currency,
            income_type: None,
            category_id: None,
            to_wallet_id: None,
            to_amount: None,
            to_currency: None,
            holding_id: Some(holding_id.to_string()),
            asset_quantity: Some(quantity),
            asset_price: Some(price),
            note: None,
            txn_date,
        }
    }

    pub fn invest_sell(wallet_id: &str, holding_id: &str, quantity: Decimal, price: Decimal, currency: Currency, txn_date: i64) -> Self {
        let total = quantity * price;
        Self {
            txn_type: TransactionType::InvestSell,
            wallet_id: wallet_id.to_string(),
            amount: total,
            currency,
            income_type: None,
            category_id: None,
            to_wallet_id: None,
            to_amount: None,
            to_currency: None,
            holding_id: Some(holding_id.to_string()),
            asset_quantity: Some(quantity),
            asset_price: Some(price),
            note: None,
            txn_date,
        }
    }
}
