use rusqlite::Row;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use super::{decimal_to_f64, f64_to_decimal};
use crate::error::{AppError, Result};

// ─── Currency ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    VND,
    USD,
    GOLD,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Currency::VND  => write!(f, "VND"),
            Currency::USD  => write!(f, "USD"),
            Currency::GOLD => write!(f, "GOLD"),
        }
    }
}

impl FromStr for Currency {
    type Err = AppError;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "VND"  => Ok(Currency::VND),
            "USD"  => Ok(Currency::USD),
            "GOLD" => Ok(Currency::GOLD),
            _      => Err(AppError::Parse(format!("Unknown currency: {s}"))),
        }
    }
}

// ─── WalletType ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WalletType {
    Cash,
    Bank,
    Loan,
    EWallet,
    Investment,
}

impl fmt::Display for WalletType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WalletType::Cash       => write!(f, "cash"),
            WalletType::Bank       => write!(f, "bank"),
            WalletType::Loan       => write!(f, "loan"),
            WalletType::EWallet    => write!(f, "e_wallet"),
            WalletType::Investment => write!(f, "investment"),
        }
    }
}

impl FromStr for WalletType {
    type Err = AppError;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "cash"       => Ok(WalletType::Cash),
            "bank"       => Ok(WalletType::Bank),
            "loan"       => Ok(WalletType::Loan),
            "e_wallet"   => Ok(WalletType::EWallet),
            "investment" => Ok(WalletType::Investment),
            _            => Err(AppError::Parse(format!("Unknown wallet type: {s}"))),
        }
    }
}

// ─── Wallet ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wallet {
    pub id:          String,
    pub name:        String,
    pub wallet_type: WalletType,
    pub currency:    Currency,
    pub balance:     Decimal,
    pub broker:      Option<String>,
    pub icon:        Option<String>,
    pub color:       Option<String>,
    pub is_active:   bool,
    pub sort_order:  i32,
    pub created_at:  i64,
}

impl Wallet {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let balance_f64: f64 = row.get("balance")?;
        let is_active_i: i32 = row.get("is_active")?;
        let wt: String = row.get("wallet_type")?;
        let cur: String = row.get("currency")?;

        Ok(Wallet {
            id:          row.get("id")?,
            name:        row.get("name")?,
            wallet_type: WalletType::from_str(&wt).map_err(to_sqlite_err)?,
            currency:    Currency::from_str(&cur).map_err(to_sqlite_err)?,
            balance:     f64_to_decimal(balance_f64),
            broker:      row.get("broker")?,
            icon:        row.get("icon")?,
            color:       row.get("color")?,
            is_active:   is_active_i != 0,
            sort_order:  row.get("sort_order")?,
            created_at:  row.get("created_at")?,
        })
    }

    pub fn balance_f64(&self) -> f64 {
        decimal_to_f64(self.balance)
    }
}

// ─── NewWallet (for inserts) ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewWallet {
    pub name:        String,
    pub wallet_type: WalletType,
    pub currency:    Currency,
    pub balance:     Decimal,
    pub broker:      Option<String>,
    pub icon:        Option<String>,
    pub color:       Option<String>,
    pub sort_order:  i32,
}

// ─── Helper ───────────────────────────────────────────────────────────────────

pub(crate) fn to_sqlite_err(e: AppError) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        rusqlite::types::Type::Text,
        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())),
    )
}
