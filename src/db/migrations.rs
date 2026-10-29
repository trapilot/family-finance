use rusqlite::{Connection, Result};

pub fn run(conn: &Connection) -> Result<()> {
    let version = get_version(conn)?;
    if version < 1 {
        migrate_v1(conn)?;
        set_version(conn, 1)?;
    }
    if version < 2 {
        migrate_v2(conn)?;
        set_version(conn, 2)?;
    }
    if version < 3 {
        migrate_v3(conn)?;
        set_version(conn, 3)?;
    }
    if version < 4 {
        migrate_v4(conn)?;
        set_version(conn, 4)?;
    }
    if version < 5 {
        migrate_v5(conn)?;
        set_version(conn, 5)?;
    }
    if version < 6 {
        migrate_v6(conn)?;
        set_version(conn, 6)?;
    }
    if version < 7 {
        migrate_v7(conn)?;
        set_version(conn, 7)?;
    }
    if version < 8 {
        migrate_v8(conn)?;
        set_version(conn, 8)?;
    }
    if version < 9 {
        migrate_v9(conn)?;
        set_version(conn, 9)?;
    }
    Ok(())
}

fn get_version(conn: &Connection) -> Result<u32> {
    conn.query_row("PRAGMA user_version", [], |row| row.get(0))
}

fn set_version(conn: &Connection, version: u32) -> Result<()> {
    conn.execute_batch(&format!("PRAGMA user_version = {version}"))
}

// ─── V1 — initial schema ──────────────────────────────────────────────────────

fn migrate_v1(conn: &Connection) -> Result<()> {
    let _ = conn.execute_batch("
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS wallets (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            wallet_type TEXT NOT NULL,
            currency    TEXT NOT NULL,
            balance     REAL NOT NULL DEFAULT 0,
            broker      TEXT,
            icon        TEXT,
            color       TEXT,
            is_active   INTEGER NOT NULL DEFAULT 1,
            sort_order  INTEGER NOT NULL DEFAULT 0,
            created_at  INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS holdings (
            id            TEXT PRIMARY KEY,
            wallet_id     TEXT NOT NULL REFERENCES wallets(id),
            symbol        TEXT NOT NULL,
            name          TEXT,
            asset_type    TEXT NOT NULL,
            quantity      REAL NOT NULL DEFAULT 0,
            avg_buy_price REAL NOT NULL DEFAULT 0,
            last_price    REAL,
            last_price_at INTEGER,
            created_at    INTEGER NOT NULL,
            UNIQUE(wallet_id, symbol)
        );

        CREATE TABLE IF NOT EXISTS categories (
            id            TEXT PRIMARY KEY,
            name          TEXT NOT NULL,
            icon          TEXT,
            color         TEXT,
            budget_amount REAL,
            parent_id     TEXT REFERENCES categories(id),
            sort_order    INTEGER DEFAULT 0,
            is_system     INTEGER DEFAULT 0,
            created_at    INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS transactions (
            id             TEXT PRIMARY KEY,
            txn_type       TEXT NOT NULL,
            wallet_id      TEXT NOT NULL REFERENCES wallets(id),
            amount         REAL NOT NULL,
            currency       TEXT NOT NULL,
            income_type    TEXT,
            category_id    TEXT REFERENCES categories(id),
            to_wallet_id   TEXT REFERENCES wallets(id),
            to_amount      REAL,
            to_currency    TEXT,
            holding_id     TEXT REFERENCES holdings(id),
            asset_quantity REAL,
            asset_price    REAL,
            note           TEXT,
            txn_date       INTEGER NOT NULL,
            created_at     INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
    ");
    
    let _ = conn.execute_batch("
        CREATE INDEX IF NOT EXISTS idx_txn_date      ON transactions(txn_date DESC);
        CREATE INDEX IF NOT EXISTS idx_txn_wallet    ON transactions(wallet_id);
        CREATE INDEX IF NOT EXISTS idx_txn_category  ON transactions(category_id);
        CREATE INDEX IF NOT EXISTS idx_txn_type      ON transactions(txn_type);
        CREATE INDEX IF NOT EXISTS idx_holding_wallet ON holdings(wallet_id);
    ");

    Ok(())
}

// ─── V2 — Phase 2: add members ────────

fn migrate_v2(conn: &Connection) -> Result<()> {
    let _ = conn.execute_batch("
        CREATE TABLE IF NOT EXISTS members (
            id             TEXT PRIMARY KEY,
            full_name      TEXT NOT NULL,
            birth_date     INTEGER,
            gender         TEXT,
            phone          TEXT,
            id_number      TEXT,
            id_issue_date  INTEGER,
            id_issue_place TEXT,
            address        TEXT,
            role           TEXT NOT NULL DEFAULT 'member',
            avatar_emoji   TEXT,
            note           TEXT,
            created_at     INTEGER NOT NULL
        );
    ");

    Ok(())
}

// ─── V3 — Phase 3: add families ────────

fn migrate_v3(conn: &Connection) -> Result<()> {
    let _ = conn.execute_batch("
        CREATE TABLE IF NOT EXISTS families (
            id             TEXT PRIMARY KEY,
            name           TEXT NOT NULL,
            common_address TEXT,
            note           TEXT,
            created_at     INTEGER NOT NULL
        );

        ALTER TABLE members ADD COLUMN family_id TEXT REFERENCES families(id);

        CREATE INDEX IF NOT EXISTS idx_member_family ON members(family_id);
    ");

    Ok(())
}

// ─── V4 — app settings: app version + UI language ────────

fn migrate_v4(conn: &Connection) -> Result<()> {
    // INSERT OR IGNORE keeps a language the user already picked
    conn.execute_batch("
        INSERT OR IGNORE INTO settings (key, value) VALUES ('app_version', '');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('language', 'en');
    ")
}

// ─── V5 — exchange rates (latest value per currency, fetched online) ────────

fn migrate_v5(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS exchange_rates (
            currency   TEXT PRIMARY KEY,   -- 'USD' / 'GOLD'
            rate_vnd   REAL NOT NULL,      -- VND for 1 unit (1 USD, 1 chỉ gold)
            source     TEXT NOT NULL,      -- where it came from, e.g. 'yahoo'
            fetched_at INTEGER NOT NULL    -- unix seconds
        );
    ")
}

// ─── V6 — recurring income/transfer reminders ────────

fn migrate_v6(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS reminders (
            id           TEXT PRIMARY KEY,
            txn_type     TEXT NOT NULL,              -- 'income' / 'transfer'
            wallet_id    TEXT NOT NULL REFERENCES wallets(id),
            to_wallet_id TEXT REFERENCES wallets(id), -- transfer only
            amount       REAL NOT NULL,
            currency     TEXT NOT NULL,
            note         TEXT,
            start_date   INTEGER NOT NULL,           -- first due day (unix seconds, local noon)
            repeat_unit  TEXT NOT NULL,              -- 'day' / 'week' / 'month' / 'year'
            repeat_every INTEGER NOT NULL DEFAULT 1,
            created_at   INTEGER NOT NULL
        );

        -- On a due day, notify every N hours
        INSERT OR IGNORE INTO settings (key, value) VALUES ('reminder_interval_hours', '4');
    ")
}

// ─── V7 — stop reminder notifications: done for a day / paused; first notification time ────────

fn migrate_v7(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        ALTER TABLE reminders ADD COLUMN done_on TEXT;                        -- 'YYYY-MM-DD' marked done
        ALTER TABLE reminders ADD COLUMN is_paused INTEGER NOT NULL DEFAULT 0;

        INSERT OR IGNORE INTO settings (key, value) VALUES ('reminder_start_time', '08:00');
    ")
}

// ─── V8 — CCCD photos (front / back), one JPEG per side ────────

fn migrate_v8(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS member_id_images (
            member_id  TEXT NOT NULL REFERENCES members(id),
            side       TEXT NOT NULL,                -- 'front' / 'back'
            jpeg       BLOB NOT NULL,
            updated_at INTEGER NOT NULL,
            PRIMARY KEY (member_id, side)
        );
    ")
}

// ─── V9 — monthly bills: notify from the due day until that month is marked paid ────────

fn migrate_v9(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS bills (
            id         TEXT PRIMARY KEY,
            name       TEXT NOT NULL,
            amount     REAL,                       -- optional, VND
            due_day    INTEGER NOT NULL,           -- 1..31, short months use their last day
            paid_month TEXT,                       -- 'YYYY-MM' marked paid
            is_paused  INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL
        );
    ")
}
