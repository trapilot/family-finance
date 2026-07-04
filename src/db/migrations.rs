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