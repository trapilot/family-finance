use chrono::{Datelike, Local, Months, NaiveDate, TimeZone};
use rusqlite::Row;
use rust_decimal::Decimal;

use super::f64_to_decimal;

// ─── Bill ─────────────────────────────────────────────────────────────────────

/// A bill paid every month. From `due_day` it notifies every day until the user marks
/// that month paid; the next month starts again on `due_day`.
#[derive(Debug, Clone, PartialEq)]
pub struct Bill {
    pub id:         String,
    pub name:       String,
    /// Optional VND amount, shown in the notification
    pub amount:     Option<Decimal>,
    /// 1–31; short months use their last day
    pub due_day:    u32,
    /// Month ("YYYY-MM") marked paid
    pub paid_month: Option<String>,
    /// Paused bills never notify
    pub is_paused:  bool,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct NewBill {
    pub name:    String,
    pub amount:  Option<Decimal>,
    pub due_day: u32,
}

impl Bill {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let amount: Option<f64> = row.get("amount")?;
        let due_day: i64        = row.get("due_day")?;
        let paused: i32         = row.get("is_paused")?;
        Ok(Bill {
            id:         row.get("id")?,
            name:       row.get("name")?,
            amount:     amount.map(f64_to_decimal),
            due_day:    due_day.clamp(1, 31) as u32,
            paid_month: row.get("paid_month")?,
            is_paused:  paused != 0,
            created_at: row.get("created_at")?,
        })
    }

    /// The due day in the month of `any_day`, e.g. day 31 → 30 Nov / 28 Feb
    pub fn due_in_month(&self, any_day: NaiveDate) -> NaiveDate {
        let first = any_day.with_day(1).unwrap_or(any_day);
        let last = (first + Months::new(1)).pred_opt().unwrap_or(first);
        first.with_day(self.due_day.min(last.day())).unwrap_or(last)
    }

    /// Due day of the month whose bill is open on `date`: this month's once it has
    /// passed, otherwise last month's (an unpaid bill keeps going into the next month).
    pub fn cycle_due(&self, date: NaiveDate) -> NaiveDate {
        let this_month = self.due_in_month(date);
        if date >= this_month {
            this_month
        } else {
            self.due_in_month(date - Months::new(1))
        }
    }

    /// "YYYY-MM" of the cycle open on `date` (the value stored in `paid_month`)
    pub fn cycle_key(&self, date: NaiveDate) -> String {
        self.cycle_due(date).format("%Y-%m").to_string()
    }

    /// Whether to notify on `date`: not paused, not paid, and the cycle started after the
    /// bill was added (a bill added on the 20th with due day 5 first notifies next month).
    pub fn is_unpaid_on(&self, date: NaiveDate) -> bool {
        let due = self.cycle_due(date);
        !self.is_paused
            && due >= self.created_day()
            && self.paid_month.as_deref() != Some(self.cycle_key(date).as_str())
    }

    /// First day on or after `from` that notifies, unless the bill is paused
    pub fn next_due(&self, from: NaiveDate) -> Option<NaiveDate> {
        if self.is_paused {
            return None;
        }
        from.iter_days().take(70).find(|d| self.is_unpaid_on(*d))
    }

    fn created_day(&self) -> NaiveDate {
        Local.timestamp_opt(self.created_at, 0).single()
            .map(|dt| dt.date_naive())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate { NaiveDate::from_ymd_opt(y, m, day).unwrap() }

    fn bill(due_day: u32, created: NaiveDate) -> Bill {
        let ts = Local.from_local_datetime(&created.and_hms_opt(9, 0, 0).unwrap()).single().unwrap().timestamp();
        Bill { id: "b".into(), name: "Electricity".into(), amount: None, due_day, paid_month: None, is_paused: false, created_at: ts }
    }

    #[test]
    fn notifies_from_due_day_until_paid_then_next_month_again() {
        let mut b = bill(5, d(2026, 10, 1));
        assert!(!b.is_unpaid_on(d(2026, 10, 4)));
        assert!(b.is_unpaid_on(d(2026, 10, 5)));
        assert!(b.is_unpaid_on(d(2026, 10, 20)));

        b.paid_month = Some(b.cycle_key(d(2026, 10, 20)));
        assert_eq!(b.paid_month.as_deref(), Some("2026-10"));
        assert!(!b.is_unpaid_on(d(2026, 10, 21)));
        assert!(!b.is_unpaid_on(d(2026, 11, 4)));
        assert!(b.is_unpaid_on(d(2026, 11, 5)));
        assert_eq!(b.next_due(d(2026, 10, 21)), Some(d(2026, 11, 5)));
    }

    #[test]
    fn unpaid_keeps_going_into_next_month() {
        let b = bill(25, d(2026, 10, 1));
        assert!(b.is_unpaid_on(d(2026, 11, 3)));
        assert_eq!(b.cycle_key(d(2026, 11, 3)), "2026-10");
        assert_eq!(b.cycle_key(d(2026, 11, 25)), "2026-11");
    }

    #[test]
    fn day_31_uses_last_day_of_short_months() {
        let b = bill(31, d(2026, 1, 1));
        assert_eq!(b.due_in_month(d(2026, 2, 10)), d(2026, 2, 28));
        assert_eq!(b.due_in_month(d(2026, 4, 1)), d(2026, 4, 30));
        assert_eq!(b.cycle_key(d(2026, 3, 1)), "2026-02");
    }

    #[test]
    fn added_after_due_day_starts_next_month_and_pause_stops() {
        let mut b = bill(5, d(2026, 10, 20));
        assert!(!b.is_unpaid_on(d(2026, 10, 20)));
        assert_eq!(b.next_due(d(2026, 10, 20)), Some(d(2026, 11, 5)));
        b.is_paused = true;
        assert!(!b.is_unpaid_on(d(2026, 11, 5)));
        assert_eq!(b.next_due(d(2026, 10, 20)), None);
    }
}
