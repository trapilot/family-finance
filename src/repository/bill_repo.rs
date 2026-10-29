use rusqlite::Connection;
use uuid::Uuid;

use crate::error::Result;
use crate::models::{decimal_to_f64, Bill, NewBill};

pub struct BillRepo;

impl BillRepo {
    pub fn create(conn: &Connection, input: &NewBill) -> Result<()> {
        conn.execute(
            "INSERT INTO bills (id, name, amount, due_day, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                Uuid::new_v4().to_string(),
                input.name,
                input.amount.map(decimal_to_f64),
                input.due_day.clamp(1, 31),
                crate::views::now_ts(),
            ],
        )?;
        Ok(())
    }

    pub fn list(conn: &Connection) -> Result<Vec<Bill>> {
        let mut stmt = conn.prepare("SELECT * FROM bills ORDER BY due_day, name")?;
        let rows = stmt.query_map([], Bill::from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
    }

    /// Save name, amount, due day and paid month
    pub fn update(conn: &Connection, bill: &Bill) -> Result<()> {
        conn.execute(
            "UPDATE bills SET name = ?2, amount = ?3, due_day = ?4, paid_month = ?5 WHERE id = ?1",
            rusqlite::params![
                bill.id,
                bill.name,
                bill.amount.map(decimal_to_f64),
                bill.due_day.clamp(1, 31),
                bill.paid_month,
            ],
        )?;
        Ok(())
    }

    /// Mark a month ("YYYY-MM") paid, or `None` to undo
    pub fn set_paid_month(conn: &Connection, id: &str, month: Option<&str>) -> Result<()> {
        conn.execute("UPDATE bills SET paid_month = ?2 WHERE id = ?1", rusqlite::params![id, month])?;
        Ok(())
    }

    pub fn set_paused(conn: &Connection, id: &str, paused: bool) -> Result<()> {
        conn.execute("UPDATE bills SET is_paused = ?2 WHERE id = ?1", rusqlite::params![id, paused as i32])?;
        Ok(())
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<()> {
        conn.execute("DELETE FROM bills WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }
}
