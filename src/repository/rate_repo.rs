use rusqlite::Connection;

use crate::error::Result;
use crate::models::{decimal_to_f64, Currency, ExchangeRate};

pub struct RateRepo;

impl RateRepo {
    pub fn list(conn: &Connection) -> Result<Vec<ExchangeRate>> {
        let mut stmt = conn.prepare("SELECT * FROM exchange_rates ORDER BY currency")?;
        let rows = stmt.query_map([], ExchangeRate::from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
    }

    pub fn get(conn: &Connection, currency: &Currency) -> Result<Option<ExchangeRate>> {
        Ok(Self::list(conn)?.into_iter().find(|r| &r.currency == currency))
    }

    pub fn upsert(conn: &Connection, rate: &ExchangeRate) -> Result<()> {
        conn.execute(
            "INSERT OR REPLACE INTO exchange_rates (currency, rate_vnd, source, fetched_at)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                rate.currency.to_string(),
                decimal_to_f64(rate.rate_vnd),
                rate.source,
                rate.fetched_at,
            ],
        )?;
        Ok(())
    }
}
