pub mod dashboard;
pub mod reports;
pub mod settings;
pub mod transactions;
pub mod wallets;
pub mod members;
pub mod pin;

// ─── Shared UI helpers ────────────────────────────────────────────────────────

/// Format VND — e.g. 1.500.000 ₫
pub fn fmt_vnd(amount: rust_decimal::Decimal) -> String {
    use rust_decimal::prelude::ToPrimitive;
    let n = amount.to_f64().unwrap_or(0.0) as i64;
    let s = format_thousands(n);
    format!("{s} ₫")
}

/// Format USD
pub fn fmt_usd(amount: rust_decimal::Decimal) -> String {
    use rust_decimal::prelude::ToPrimitive;
    let n = amount.to_f64().unwrap_or(0.0);
    format!("${n:.2}")
}

/// Format GOLD — e.g. 2.50 chỉ
pub fn fmt_gold(amount: rust_decimal::Decimal) -> String {
    use rust_decimal::prelude::ToPrimitive;
    let n = amount.to_f64().unwrap_or(0.0);
    format!("{n:.3} chỉ")
}

pub fn fmt_currency(amount: rust_decimal::Decimal, currency: &crate::models::Currency) -> String {
    match currency {
        crate::models::Currency::VND  => fmt_vnd(amount),
        crate::models::Currency::USD  => fmt_usd(amount),
        crate::models::Currency::GOLD => fmt_gold(amount),
    }
}

fn format_thousands(n: i64) -> String {
    let s = n.abs().to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { result.push('.'); }
        result.push(c);
    }
    let mut out: String = result.chars().rev().collect();
    if n < 0 { out = format!("-{out}"); }
    out
}

/// Current month/year as (year: i32, month: u32)
pub fn current_month() -> (i32, u32) {
    use chrono::{Datelike, Local};
    let now = Local::now();
    (now.year(), now.month())
}

pub fn month_label(year: i32, month: u32) -> String {
    let names = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
    let name = names.get((month - 1) as usize).unwrap_or(&"?");
    format!("{name} {year}")
}

pub fn now_ts() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

pub fn ts_to_date_str(ts: i64) -> String {
    use chrono::{Local, TimeZone};
    let dt = Local.timestamp_opt(ts, 0).single().unwrap_or_else(|| Local::now());
    dt.format("%d/%m/%Y").to_string()
}

pub fn date_str_to_ts(s: &str) -> Option<i64> {
    use chrono::{Local, NaiveDate, TimeZone};
    let d = NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()?;
    Some(Local.from_local_datetime(&d.and_hms_opt(12, 0, 0)?).single()?.timestamp())
}
