use chrono::{Datelike, Local, Months, NaiveDate, TimeZone};
use rusqlite::Row;
use rust_decimal::Decimal;
use std::fmt;
use std::str::FromStr;

use super::{f64_to_decimal, wallet::to_sqlite_err, Currency, TransactionType};
use crate::error::{AppError, Result};

// ─── RepeatUnit ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatUnit {
    Day,
    Week,
    Month,
    Year,
}

impl RepeatUnit {
    pub const ALL: [RepeatUnit; 4] = [RepeatUnit::Day, RepeatUnit::Week, RepeatUnit::Month, RepeatUnit::Year];
}

impl fmt::Display for RepeatUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepeatUnit::Day   => write!(f, "day"),
            RepeatUnit::Week  => write!(f, "week"),
            RepeatUnit::Month => write!(f, "month"),
            RepeatUnit::Year  => write!(f, "year"),
        }
    }
}

impl FromStr for RepeatUnit {
    type Err = AppError;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "day"   => Ok(RepeatUnit::Day),
            "week"  => Ok(RepeatUnit::Week),
            "month" => Ok(RepeatUnit::Month),
            "year"  => Ok(RepeatUnit::Year),
            _       => Err(AppError::Parse(format!("Unknown repeat unit: {s}"))),
        }
    }
}

// ─── Reminder ─────────────────────────────────────────────────────────────────

/// A recurring income/transfer the user expects; only used to notify, never to add transactions.
#[derive(Debug, Clone, PartialEq)]
pub struct Reminder {
    pub id:           String,
    pub txn_type:     TransactionType,
    pub wallet_id:    String,
    pub to_wallet_id: Option<String>,
    pub amount:       Decimal,
    pub currency:     Currency,
    pub note:         Option<String>,
    pub start_date:   i64,
    pub repeat_unit:  RepeatUnit,
    pub repeat_every: u32,
    pub created_at:   i64,
    /// Day ("YYYY-MM-DD") the user marked done; no more notifications that day
    pub done_on:      Option<String>,
    /// Paused reminders never notify
    pub is_paused:    bool,
}

#[derive(Debug, Clone)]
pub struct NewReminder {
    pub txn_type:     TransactionType,
    pub wallet_id:    String,
    pub to_wallet_id: Option<String>,
    pub amount:       Decimal,
    pub currency:     Currency,
    pub note:         Option<String>,
    pub start_date:   i64,
    pub repeat_unit:  RepeatUnit,
    pub repeat_every: u32,
}

impl Reminder {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let txn_type: String = row.get("txn_type")?;
        let currency: String = row.get("currency")?;
        let unit: String     = row.get("repeat_unit")?;
        let every: i64       = row.get("repeat_every")?;
        let paused: i32      = row.get("is_paused")?;
        Ok(Reminder {
            id:           row.get("id")?,
            txn_type:     TransactionType::from_str(&txn_type).map_err(to_sqlite_err)?,
            wallet_id:    row.get("wallet_id")?,
            to_wallet_id: row.get("to_wallet_id")?,
            amount:       f64_to_decimal(row.get("amount")?),
            currency:     Currency::from_str(&currency).map_err(to_sqlite_err)?,
            note:         row.get("note")?,
            start_date:   row.get("start_date")?,
            repeat_unit:  RepeatUnit::from_str(&unit).map_err(to_sqlite_err)?,
            repeat_every: every.max(1) as u32,
            created_at:   row.get("created_at")?,
            done_on:      row.get("done_on")?,
            is_paused:    paused != 0,
        })
    }

    pub fn start_day(&self) -> NaiveDate {
        Local.timestamp_opt(self.start_date, 0).single()
            .map(|dt| dt.date_naive())
            .unwrap_or_default()
    }

    /// Whether the reminder falls on `date` (start day, then every N units).
    pub fn is_due(&self, date: NaiveDate) -> bool {
        let start = self.start_day();
        if date < start {
            return false;
        }
        let every = self.repeat_every.max(1) as i64;
        let days = (date - start).num_days();
        match self.repeat_unit {
            RepeatUnit::Day   => days % every == 0,
            RepeatUnit::Week  => days % (7 * every) == 0,
            RepeatUnit::Month => month_step_matches(start, date, every),
            RepeatUnit::Year  => month_step_matches(start, date, 12 * every),
        }
    }

    /// Whether notifications for `date` were stopped with "Done for today"
    pub fn is_done_on(&self, date: NaiveDate) -> bool {
        self.done_on.as_deref() == Some(date.format("%Y-%m-%d").to_string().as_str())
    }

    /// First due day on or after `from` (searches up to ~10 years ahead).
    pub fn next_due(&self, from: NaiveDate) -> Option<NaiveDate> {
        let from = from.max(self.start_day());
        from.iter_days().take(3700).find(|d| self.is_due(*d))
    }
}

/// Counting months from `start` (never from a clamped date, so the 31st stays the 31st
/// where it exists), is `date` exactly `k * step` months after `start`?
fn month_step_matches(start: NaiveDate, date: NaiveDate, step: i64) -> bool {
    let months = (date.year() - start.year()) as i64 * 12 + date.month0() as i64 - start.month0() as i64;
    months >= 0
        && months % step == 0
        && start.checked_add_months(Months::new(months as u32)) == Some(date)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reminder(start: NaiveDate, unit: RepeatUnit, every: u32) -> Reminder {
        let ts = Local.from_local_datetime(&start.and_hms_opt(12, 0, 0).unwrap()).single().unwrap().timestamp();
        Reminder {
            id: "r".into(), txn_type: TransactionType::Income, wallet_id: "w".into(), to_wallet_id: None,
            amount: Decimal::ONE, currency: Currency::VND, note: None, start_date: ts,
            repeat_unit: unit, repeat_every: every, created_at: 0, done_on: None, is_paused: false,
        }
    }
    fn d(y: i32, m: u32, day: u32) -> NaiveDate { NaiveDate::from_ymd_opt(y, m, day).unwrap() }

    #[test]
    fn monthly_keeps_the_day_and_clamps_short_months() {
        let r = reminder(d(2026, 1, 31), RepeatUnit::Month, 1);
        assert!(r.is_due(d(2026, 1, 31)));
        assert!(r.is_due(d(2026, 2, 28)));   // clamped
        assert!(r.is_due(d(2026, 3, 31)));   // back to the 31st
        assert!(!r.is_due(d(2026, 3, 28)));
        assert!(!r.is_due(d(2025, 12, 31))); // before start
    }

    #[test]
    fn every_two_weeks_and_days() {
        let r = reminder(d(2026, 10, 1), RepeatUnit::Week, 2);
        assert!(r.is_due(d(2026, 10, 15)));
        assert!(!r.is_due(d(2026, 10, 8)));
        let r = reminder(d(2026, 10, 1), RepeatUnit::Day, 3);
        assert!(r.is_due(d(2026, 10, 4)));
        assert!(!r.is_due(d(2026, 10, 5)));
    }

    #[test]
    fn yearly_and_next_due() {
        let r = reminder(d(2024, 2, 29), RepeatUnit::Year, 1);
        assert!(r.is_due(d(2025, 2, 28)));
        assert_eq!(r.next_due(d(2026, 3, 1)), Some(d(2027, 2, 28)));
    }
}
