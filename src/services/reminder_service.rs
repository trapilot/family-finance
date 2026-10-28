use chrono::{DateTime, Duration, Local, NaiveDate, TimeZone};
use rusqlite::Connection;

use crate::error::Result;
use crate::i18n::{translate, Lang};
use crate::models::{Reminder, TransactionType, Wallet};
use crate::notify::{self, ScheduledNotification};
use crate::repository::{ReminderRepo, SettingsRepo, WalletRepo};
use crate::views::fmt_currency;

/// How far ahead notifications are scheduled (re-synced every time the app opens)
const SCHEDULE_DAYS: i64 = 14;
/// iOS keeps at most 64 pending notifications per app
const MAX_PENDING: usize = 60;

pub struct ReminderService;

impl ReminderService {
    /// Active (not paused) reminders due on `date`, including ones already marked done
    pub fn due_on(conn: &Connection, date: NaiveDate) -> Result<Vec<Reminder>> {
        Ok(ReminderRepo::list(conn)?.into_iter().filter(|r| !r.is_paused && r.is_due(date)).collect())
    }

    /// Notification times of a due day as minutes after midnight:
    /// the configured first time, then every `interval_hours` until midnight.
    pub fn day_slots(conn: &Connection) -> Result<Vec<u32>> {
        let (hour, minute) = SettingsRepo::get_reminder_start_time(conn)?;
        let interval = SettingsRepo::get_reminder_interval_hours(conn)?;
        Ok((hour * 60 + minute..24 * 60).step_by((interval * 60) as usize).collect())
    }

    fn notification_id(reminder_id: &str, day: NaiveDate, slot: u32) -> String {
        format!("reminder:{reminder_id}:{day}:{slot}")
    }

    /// Stop the rest of `day`'s notifications for one reminder ("Done for today").
    pub fn mark_done(conn: &Connection, reminder_id: &str, day: NaiveDate) -> Result<()> {
        ReminderRepo::mark_done(conn, reminder_id, &day.format("%Y-%m-%d").to_string())?;
        // Also clear the ones already shown in Notification Center
        let delivered: Vec<String> = Self::day_slots(conn)?.into_iter()
            .map(|slot| Self::notification_id(reminder_id, day, slot))
            .collect();
        notify::clear_delivered(&delivered);
        Self::resync(conn)
    }

    /// Handle the "Done for today" action of a notification with id `reminder:<id>:<day>:<slot>`.
    pub fn handle_done_action(conn: &Connection, notification_id: &str) -> Result<()> {
        let mut parts = notification_id.split(':');
        let (Some("reminder"), Some(id), Some(day)) = (parts.next(), parts.next(), parts.next()) else {
            return Ok(());
        };
        match NaiveDate::parse_from_str(day, "%Y-%m-%d") {
            Ok(day) => Self::mark_done(conn, id, day),
            Err(_)  => Ok(()),
        }
    }

    /// Notification title and body for a reminder, e.g. ("💰 Expected income today", "Vietcombank · +15.000.000 ₫ — Salary")
    pub fn describe(reminder: &Reminder, wallets: &[Wallet], lang: Lang) -> (String, String) {
        let wallet_name = |id: &str| wallets.iter().find(|w| w.id == id).map(|w| w.name.clone()).unwrap_or_default();
        let amount = fmt_currency(reminder.amount, &reminder.currency);

        let (title, mut body) = match reminder.txn_type {
            TransactionType::Transfer => {
                let to = reminder.to_wallet_id.as_deref().map(wallet_name).unwrap_or_default();
                (translate(lang, "⇄ Transfer due today"), format!("{} → {to} · {amount}", wallet_name(&reminder.wallet_id)))
            }
            _ => (translate(lang, "💰 Expected income today"), format!("{} · +{amount}", wallet_name(&reminder.wallet_id))),
        };
        if let Some(note) = reminder.note.as_deref().filter(|n| !n.is_empty()) {
            body.push_str(&format!(" — {note}"));
        }
        (title.to_string(), body)
    }

    /// Every notification for the next `SCHEDULE_DAYS` days: on each due day, one per reminder
    /// at the first notification time and then every `interval_hours` until midnight.
    /// Paused reminders and days marked done are skipped.
    pub fn build_schedule(conn: &Connection, now: DateTime<Local>) -> Result<Vec<ScheduledNotification>> {
        let reminders = ReminderRepo::list(conn)?;
        if reminders.is_empty() {
            return Ok(vec![]);
        }
        let wallets  = WalletRepo::list_active(conn)?;
        let slots    = Self::day_slots(conn)?;
        let lang     = SettingsRepo::get_language(conn)?.map(|c| Lang::from_code(&c)).unwrap_or_default();

        let mut items = vec![];
        let today = now.date_naive();
        for offset in 0..SCHEDULE_DAYS {
            let day = today + Duration::days(offset);
            let due: Vec<&Reminder> = reminders.iter()
                .filter(|r| !r.is_paused && r.is_due(day) && !r.is_done_on(day))
                .collect();
            if due.is_empty() {
                continue;
            }
            for &slot in &slots {
                let Some(at) = day.and_hms_opt(slot / 60, slot % 60, 0).and_then(|t| Local.from_local_datetime(&t).single()) else { continue };
                if at <= now {
                    continue;
                }
                for reminder in &due {
                    let (title, body) = Self::describe(reminder, &wallets, lang);
                    items.push(ScheduledNotification {
                        id: Self::notification_id(&reminder.id, day, slot),
                        title,
                        body,
                        fire_at: at.timestamp(),
                    });
                }
            }
        }
        items.sort_by_key(|n| n.fire_at);
        items.truncate(MAX_PENDING);
        Ok(items)
    }

    /// Replace the app's pending system notifications with the current schedule.
    /// Call after reminders, the interval or the language change, and when the app opens.
    pub fn resync(conn: &Connection) -> Result<()> {
        let now = Local::now();
        let items = Self::build_schedule(conn, now)?;
        let lang = SettingsRepo::get_language(conn)?.map(|c| Lang::from_code(&c)).unwrap_or_default();
        notify::replace_all(&items, now.timestamp(), translate(lang, "✓ Done for today"));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Currency, NewReminder, NewWallet, RepeatUnit, WalletType};
    use rust_decimal::Decimal;

    fn setup(interval: u32) -> (Connection, String) {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&conn).unwrap();
        let wallet = WalletRepo::create(&conn, &NewWallet {
            name: "Vietcombank".into(), wallet_type: WalletType::Bank, currency: Currency::VND,
            balance: Decimal::ZERO, broker: None, icon: None, color: None, sort_order: 0,
        }).unwrap();
        SettingsRepo::set_reminder_interval_hours(&conn, interval).unwrap();
        (conn, wallet.id)
    }

    fn add_daily(conn: &Connection, wallet_id: &str, start: DateTime<Local>) {
        ReminderRepo::create(conn, &NewReminder {
            txn_type: TransactionType::Income, wallet_id: wallet_id.into(), to_wallet_id: None,
            amount: Decimal::from(15_000_000), currency: Currency::VND, note: Some("Salary".into()),
            start_date: start.timestamp(), repeat_unit: RepeatUnit::Day, repeat_every: 1,
        }).unwrap();
    }

    fn at(day: NaiveDate, hour: u32) -> DateTime<Local> {
        Local.from_local_datetime(&day.and_hms_opt(hour, 0, 0).unwrap()).single().unwrap()
    }

    #[test]
    fn notifies_from_8am_every_interval_until_midnight() {
        let (conn, wallet) = setup(4);
        let day = NaiveDate::from_ymd_opt(2026, 11, 2).unwrap();
        add_daily(&conn, &wallet, at(day, 12));

        // At 06:00 on the start day: 08, 12, 16, 20 today
        let items = ReminderService::build_schedule(&conn, at(day, 6)).unwrap();
        let today: Vec<i64> = items.iter().filter(|n| n.fire_at < at(day, 23).timestamp()).map(|n| n.fire_at).collect();
        assert_eq!(today, [8, 12, 16, 20].map(|h| at(day, h).timestamp()));
        assert_eq!(items[0].title, "💰 Expected income today");
        assert_eq!(items[0].body, "Vietcombank · +15.000.000 ₫ — Salary");
    }

    #[test]
    fn skips_past_slots_and_caps_at_ios_limit() {
        let (conn, wallet) = setup(1);
        let day = NaiveDate::from_ymd_opt(2026, 11, 2).unwrap();
        add_daily(&conn, &wallet, at(day, 12));

        let now = at(day, 9) + chrono::Duration::minutes(30);
        let items = ReminderService::build_schedule(&conn, now).unwrap();
        assert!(items.iter().all(|n| n.fire_at > now.timestamp()));
        assert_eq!(items.len(), MAX_PENDING);
        assert_eq!(items[0].fire_at, at(day, 10).timestamp());
    }

    #[test]
    fn custom_start_time_done_and_paused() {
        let (conn, wallet) = setup(6);
        SettingsRepo::set_reminder_start_time(&conn, 7, 30).unwrap();
        let day = NaiveDate::from_ymd_opt(2026, 11, 2).unwrap();
        add_daily(&conn, &wallet, at(day, 12));
        let now = at(day, 6);

        let first_day = |items: &[ScheduledNotification]| -> Vec<i64> {
            items.iter().filter(|n| n.fire_at < at(day, 23).timestamp() + 3600).map(|n| n.fire_at).collect()
        };
        let items = ReminderService::build_schedule(&conn, now).unwrap();
        let expect: Vec<i64> = [(7, 30), (13, 30), (19, 30)].iter()
            .map(|(h, m)| (at(day, *h) + chrono::Duration::minutes(*m)).timestamp()).collect();
        assert_eq!(first_day(&items), expect);

        // "Done for today" from the notification stops the rest of that day only
        ReminderService::handle_done_action(&conn, &items[0].id).unwrap();
        let items = ReminderService::build_schedule(&conn, now).unwrap();
        assert!(first_day(&items).is_empty());
        assert!(!items.is_empty(), "next days still notify");

        // Pausing stops everything
        let id = ReminderRepo::list(&conn).unwrap()[0].id.clone();
        ReminderRepo::set_paused(&conn, &id, true).unwrap();
        assert!(ReminderService::build_schedule(&conn, now).unwrap().is_empty());
    }

    #[test]
    fn nothing_scheduled_without_reminders() {
        let (conn, _) = setup(4);
        assert!(ReminderService::build_schedule(&conn, Local::now()).unwrap().is_empty());
    }
}
