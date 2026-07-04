use crate::error::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use rusqlite::{Connection, OptionalExtension};

pub struct SettingsRepo;

impl SettingsRepo {
    const KEY_PIN_HASH: &'static str = "pin_hash";

    pub fn set_pin(conn: &Connection, pin: &str) -> Result<()> {
        let hashed_pin = hash(pin, DEFAULT_COST)?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            rusqlite::params![Self::KEY_PIN_HASH, &hashed_pin],
        )?;
        Ok(())
    }

    pub fn verify_pin(conn: &Connection, pin: &str) -> Result<bool> {
        let hash: Option<String> = conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            rusqlite::params![Self::KEY_PIN_HASH],
            |row| row.get(0),
        ).optional()?;

        match hash {
            Some(hash_str) => Ok(verify(pin, &hash_str)?),
            None => Ok(false),
        }
    }

    pub fn is_pin_set(conn: &Connection) -> Result<bool> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM settings WHERE key = ?1",
            rusqlite::params![Self::KEY_PIN_HASH],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn clear_pin(conn: &Connection) -> Result<()> {
        conn.execute(
            "DELETE FROM settings WHERE key = ?1",
            rusqlite::params![Self::KEY_PIN_HASH],
        )?;
        Ok(())
    }
}
