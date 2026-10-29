pub mod migrations;

use once_cell::sync::OnceCell;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

static DB: OnceCell<Arc<Mutex<Connection>>> = OnceCell::new();
/// File path given to `init`, needed by `reset`
static DB_PATH: OnceCell<String> = OnceCell::new();

pub fn init(path: &str) -> Arc<Mutex<Connection>> {
    let conn = Connection::open(path).expect("Failed to open SQLite database");
    migrations::run(&conn).expect("Failed to run migrations");
    let db = Arc::new(Mutex::new(conn));
    DB.set(db.clone()).ok();
    DB_PATH.set(path.to_string()).ok();
    db
}

pub fn get() -> Arc<Mutex<Connection>> {
    DB.get().expect("Database not initialized — call db::init() first").clone()
}

/// Data every database needs: system categories and the running app version
pub fn seed(conn: &Connection) -> crate::error::Result<()> {
    crate::repository::CategoryRepo::seed_defaults(conn)?;
    crate::repository::SettingsRepo::set_app_version(conn, env!("CARGO_PKG_VERSION"))
}

/// "Forgot PIN": close the database, delete its files and start over with an empty,
/// seeded one (no PIN). Backup files live elsewhere and are kept.
pub fn reset() -> crate::error::Result<()> {
    let path = DB_PATH.get()
        .ok_or_else(|| crate::error::AppError::NotFound("database path".into()))?;
    let db = get();
    let mut conn = db.lock().expect("DB lock poisoned");

    // Swap in a throwaway connection so the file is closed before deleting it
    let old = std::mem::replace(&mut *conn, Connection::open_in_memory()?);
    old.close().map_err(|(_, e)| e)?;
    for suffix in ["", "-wal", "-shm", "-journal"] {
        let file = format!("{path}{suffix}");
        if std::path::Path::new(&file).exists() {
            std::fs::remove_file(&file)?;
        }
    }

    let fresh = Connection::open(path)?;
    migrations::run(&fresh)?;
    seed(&fresh)?;
    // Reminders are gone: cancel their scheduled notifications
    crate::services::ReminderService::resync(&fresh)?;
    *conn = fresh;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::repository::{CategoryRepo, SettingsRepo};

    #[test]
    fn reset_deletes_everything_and_reseeds() {
        let dir = std::env::temp_dir().join(format!("ff-reset-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.db");
        let db = super::init(path.to_str().unwrap());
        {
            let conn = db.lock().unwrap();
            super::seed(&conn).unwrap();
            SettingsRepo::set_pin(&conn, "1234").unwrap();
            conn.execute("INSERT INTO families (id, name, created_at) VALUES ('f', 'Nguyen', 0)", []).unwrap();
        }

        super::reset().unwrap();

        let conn = db.lock().unwrap();
        assert!(!SettingsRepo::is_pin_set(&conn).unwrap());
        let families: i64 = conn.query_row("SELECT COUNT(*) FROM families", [], |r| r.get(0)).unwrap();
        assert_eq!(families, 0);
        assert!(!CategoryRepo::list(&conn).unwrap().is_empty(), "default categories re-seeded");
        assert!(path.exists(), "a fresh file is in place");
        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
