use rusqlite::Row;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use super::wallet::to_sqlite_err;
use super::{decimal_to_f64, f64_to_decimal};
use crate::error::AppError;

// ─── AssetType ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetType {
    Stock,
    Crypto,
}

impl fmt::Display for AssetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssetType::Stock  => write!(f, "stock"),
            AssetType::Crypto => write!(f, "crypto"),
        }
    }
}

impl FromStr for AssetType {
    type Err = AppError;
    fn from_str(s: &str) -> crate::error::Result<Self> {
        match s {
            "stock"  => Ok(AssetType::Stock),
            "crypto" => Ok(AssetType::Crypto),
            _        => Err(AppError::Parse(format!("Unknown asset type: {s}"))),
        }
    }
}

// ─── Holding ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Holding {
    pub id:            String,
    pub wallet_id:     String,
    pub symbol:        String,
    pub name:          Option<String>,
    pub asset_type:    AssetType,
    pub quantity:      Decimal,
    pub avg_buy_price: Decimal,
    pub last_price:    Option<Decimal>,
    pub last_price_at: Option<i64>,
    pub created_at:    i64,
}

impl Holding {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let asset_type_str: String = row.get("asset_type")?;
        let qty_f64: f64 = row.get("quantity")?;
        let avg_f64: f64 = row.get("avg_buy_price")?;
        let last_price_f64: Option<f64> = row.get("last_price")?;

        Ok(Holding {
            id:            row.get("id")?,
            wallet_id:     row.get("wallet_id")?,
            symbol:        row.get("symbol")?,
            name:          row.get("name")?,
            asset_type:    AssetType::from_str(&asset_type_str).map_err(to_sqlite_err)?,
            quantity:      f64_to_decimal(qty_f64),
            avg_buy_price: f64_to_decimal(avg_f64),
            last_price:    last_price_f64.map(f64_to_decimal),
            last_price_at: row.get("last_price_at")?,
            created_at:    row.get("created_at")?,
        })
    }

    /// Unrealized P&L using last_price
    pub fn unrealized_pnl(&self) -> Option<Decimal> {
        self.last_price.map(|lp| (lp - self.avg_buy_price) * self.quantity)
    }

    /// Market value at last_price
    pub fn market_value(&self) -> Option<Decimal> {
        self.last_price.map(|lp| lp * self.quantity)
    }

    /// Cost basis
    pub fn cost_basis(&self) -> Decimal {
        self.avg_buy_price * self.quantity
    }

    pub fn quantity_f64(&self)      -> f64 { decimal_to_f64(self.quantity) }
    pub fn avg_buy_price_f64(&self) -> f64 { decimal_to_f64(self.avg_buy_price) }
}

// ─── NewHolding ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewHolding {
    pub wallet_id:  String,
    pub symbol:     String,
    pub name:       Option<String>,
    pub asset_type: AssetType,
}
