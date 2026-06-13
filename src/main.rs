mod app;
mod db;
mod error;
mod models;
mod repository;
mod services;
mod views;

fn main() {
    let _db = db::init("family_finance.db");

    // Seed default categories on first run
    {
        let conn = _db.lock().unwrap();
        repository::CategoryRepo::seed_defaults(&conn)
            .expect("Failed to seed default categories");
    }

    dioxus::launch(app::App);
}