use rusqlite::Connection;
use uuid::Uuid;

use crate::error::Result;
use crate::models::{decimal_to_f64, NewReminder, Reminder};

pub struct ReminderRepo;

impl ReminderRepo {
    pub fn create(conn: &Connection, input: &NewReminder) -> Result<()> {
        conn.execute(
            "INSERT INTO reminders
             (id, txn_type, wallet_id, to_wallet_id, amount, currency, note, start_date, repeat_unit, repeat_every, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                Uuid::new_v4().to_string(),
                input.txn_type.to_string(),
                input.wallet_id,
                input.to_wallet_id,
                decimal_to_f64(input.amount),
                input.currency.to_string(),
                input.note,
                input.start_date,
                input.repeat_unit.to_string(),
                input.repeat_every.max(1),
                crate::views::now_ts(),
            ],
        )?;
        Ok(())
    }

    pub fn list(conn: &Connection) -> Result<Vec<Reminder>> {
        let mut stmt = conn.prepare("SELECT * FROM reminders ORDER BY start_date")?;
        let rows = stmt.query_map([], Reminder::from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
    }

    /// Stop notifications for `day` ("YYYY-MM-DD")
    pub fn mark_done(conn: &Connection, id: &str, day: &str) -> Result<()> {
        conn.execute("UPDATE reminders SET done_on = ?2 WHERE id = ?1", rusqlite::params![id, day])?;
        Ok(())
    }

    pub fn set_paused(conn: &Connection, id: &str, paused: bool) -> Result<()> {
        conn.execute("UPDATE reminders SET is_paused = ?2 WHERE id = ?1", rusqlite::params![id, paused as i32])?;
        Ok(())
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<()> {
        conn.execute("DELETE FROM reminders WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }
}
