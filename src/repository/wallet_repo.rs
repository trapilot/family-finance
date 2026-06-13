use rusqlite::Connection;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::{
    decimal_to_f64,
    wallet::{NewWallet, Wallet},
    Currency,
};

pub struct WalletRepo;

impl WalletRepo {
    pub fn create(conn: &Connection, input: &NewWallet) -> Result<Wallet> {
        let id = Uuid::new_v4().to_string();
        let now = now_ts();

        conn.execute(
            "INSERT INTO wallets
             (id, name, wallet_type, currency, balance, broker, icon, color, is_active, sort_order, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9, ?10)",
            rusqlite::params![
                id,
                input.name,
                input.wallet_type.to_string(),
                input.currency.to_string(),
                decimal_to_f64(input.balance),
                input.broker,
                input.icon,
                input.color,
                input.sort_order,
                now,
            ],
        )?;

        Self::find_by_id(conn, &id)?.ok_or_else(|| AppError::NotFound(id))
    }

    pub fn find_by_id(conn: &Connection, id: &str) -> Result<Option<Wallet>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, wallet_type, currency, balance, broker, icon, color, is_active, sort_order, created_at
             FROM wallets WHERE id = ?1",
        )?;

        let result = stmt.query_row([id], Wallet::from_row);
        match result {
            Ok(w)                                => Ok(Some(w)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e)                               => Err(e.into()),
        }
    }

    pub fn list_active(conn: &Connection) -> Result<Vec<Wallet>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, wallet_type, currency, balance, broker, icon, color, is_active, sort_order, created_at
             FROM wallets
             WHERE is_active = 1
             ORDER BY sort_order ASC, created_at ASC",
        )?;

        let rows = stmt.query_map([], Wallet::from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
    }

    pub fn list_by_currency(conn: &Connection, currency: &Currency) -> Result<Vec<Wallet>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, wallet_type, currency, balance, broker, icon, color, is_active, sort_order, created_at
             FROM wallets
             WHERE is_active = 1 AND currency = ?1
             ORDER BY sort_order ASC",
        )?;

        let rows = stmt.query_map([currency.to_string()], Wallet::from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
    }

    pub fn update(conn: &Connection, wallet: &Wallet) -> Result<()> {
        conn.execute(
            "UPDATE wallets
             SET name = ?1, wallet_type = ?2, currency = ?3, balance = ?4,
                 broker = ?5, icon = ?6, color = ?7, sort_order = ?8
             WHERE id = ?9",
            rusqlite::params![
                wallet.name,
                wallet.wallet_type.to_string(),
                wallet.currency.to_string(),
                wallet.balance_f64(),
                wallet.broker,
                wallet.icon,
                wallet.color,
                wallet.sort_order,
                wallet.id,
            ],
        )?;
        Ok(())
    }

    /// Atomic balance delta — used exclusively by TransactionService
    pub fn add_balance(conn: &Connection, wallet_id: &str, delta_f64: f64) -> Result<()> {
        let affected = conn.execute(
            "UPDATE wallets SET balance = balance + ?1 WHERE id = ?2 AND is_active = 1",
            rusqlite::params![delta_f64, wallet_id],
        )?;
        if affected == 0 {
            return Err(AppError::NotFound(format!("wallet '{wallet_id}'")));
        }
        Ok(())
    }

    pub fn soft_delete(conn: &Connection, id: &str) -> Result<()> {
        conn.execute("UPDATE wallets SET is_active = 0 WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn reorder(conn: &Connection, ordered_ids: &[String]) -> Result<()> {
        for (i, id) in ordered_ids.iter().enumerate() {
            conn.execute(
                "UPDATE wallets SET sort_order = ?1 WHERE id = ?2",
                rusqlite::params![i as i32, id],
            )?;
        }
        Ok(())
    }
}

fn now_ts() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
