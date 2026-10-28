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

    launch();
}

/// Serve our own index.html so the page fills the device screen and can respect the safe areas.
#[cfg(any(feature = "mobile", feature = "desktop"))]
fn launch() {
    // `mobile` and `desktop` both re-export dioxus_desktop
    #[cfg(feature = "mobile")]
    use dioxus::mobile::Config;
    #[cfg(not(feature = "mobile"))]
    use dioxus::desktop::Config;

    let cfg = Config::new()
        .with_custom_index(include_str!("index.html").to_string())
        .with_background_color((245, 245, 245, 255));

    dioxus::LaunchBuilder::new().with_cfg(cfg).launch(app::App);
}

#[cfg(not(any(feature = "mobile", feature = "desktop")))]
fn launch() {
    dioxus::launch(app::App);
}
