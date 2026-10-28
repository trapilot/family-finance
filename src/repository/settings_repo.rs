use crate::error::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use rusqlite::{Connection, OptionalExtension};

pub struct SettingsRepo;

impl SettingsRepo {
    const KEY_PIN_HASH: &'static str = "pin_hash";
    const KEY_LANGUAGE: &'static str = "language";
    const KEY_APP_VERSION: &'static str = "app_version";

    fn get(conn: &Connection, key: &str) -> Result<Option<String>> {
        Ok(conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            rusqlite::params![key],
            |row| row.get(0),
        ).optional()?)
    }

    fn set(conn: &Connection, key: &str, value: &str) -> Result<()> {
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, value],
        )?;
        Ok(())
    }

    /// UI language code ("en" / "vi").
    pub fn get_language(conn: &Connection) -> Result<Option<String>> {
        Self::get(conn, Self::KEY_LANGUAGE)
    }

    pub fn set_language(conn: &Connection, code: &str) -> Result<()> {
        Self::set(conn, Self::KEY_LANGUAGE, code)
    }

    /// Version of the app that last opened this database.
    pub fn get_app_version(conn: &Connection) -> Result<Option<String>> {
        Self::get(conn, Self::KEY_APP_VERSION)
    }

    /// Called on startup so the stored version always matches the running build.
    pub fn set_app_version(conn: &Connection, version: &str) -> Result<()> {
        Self::set(conn, Self::KEY_APP_VERSION, version)
    }

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
