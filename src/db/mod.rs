pub mod migrations;

use once_cell::sync::OnceCell;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

static DB: OnceCell<Arc<Mutex<Connection>>> = OnceCell::new();

pub fn init(path: &str) -> Arc<Mutex<Connection>> {
    let conn = Connection::open(path).expect("Failed to open SQLite database");
    migrations::run(&conn).expect("Failed to run migrations");
    let db = Arc::new(Mutex::new(conn));
    DB.set(db.clone()).ok();
    db
}

pub fn get() -> Arc<Mutex<Connection>> {
    DB.get().expect("Database not initialized — call db::init() first").clone()
}