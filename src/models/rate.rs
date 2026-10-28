use rusqlite::Row;
use rust_decimal::Decimal;
use std::str::FromStr;

use super::{f64_to_decimal, wallet::to_sqlite_err, Currency};

/// Latest known value of one currency in VND.
#[derive(Debug, Clone, PartialEq)]
pub struct ExchangeRate {
    pub currency:   Currency,
    pub rate_vnd:   Decimal,
    pub source:     String,
    pub fetched_at: i64,
}

impl ExchangeRate {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let cur: String = row.get("currency")?;
        Ok(Self {
            currency:   Currency::from_str(&cur).map_err(to_sqlite_err)?,
            rate_vnd:   f64_to_decimal(row.get("rate_vnd")?),
            source:     row.get("source")?,
            fetched_at: row.get("fetched_at")?,
        })
    }
}
