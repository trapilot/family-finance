mod app;
mod db;
mod error;
mod models;
mod repository;
mod services;
mod views;

use std::path::PathBuf;

fn main() {
    #[cfg(target_os = "ios")]
    let db_path = {
        let home = std::env::var_os("HOME")
            .expect("Could not find app home directory");

        let dir = PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("FamilyFinance");

        std::fs::create_dir_all(&dir)
            .expect("Failed to create database directory");

        dir.join("family_finance.db")
    };

    #[cfg(not(target_os = "ios"))]
    let db_path = PathBuf::from("family_finance.db");

    let db_path = db_path
        .to_str()
        .expect("Invalid database path");

    let db = db::init(db_path);
    {
        let conn = db.lock().unwrap();

        repository::CategoryRepo::seed_defaults(&conn)
            .expect("Failed to seed default categories");
    }

    dioxus::launch(app::App);
}
