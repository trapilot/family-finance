use rusqlite::Connection;
use rust_decimal::Decimal;

use crate::error::{AppError, Result};
use crate::models::{
    decimal_to_f64, f64_to_decimal,
    transaction::{NewTransaction, Transaction, TransactionType},
};
use crate::repository::{HoldingRepo, TransactionRepo, WalletRepo};
use crate::services::HoldingService;

pub struct TransactionService;

impl TransactionService {
    /// Single entry point for all transaction types.
    /// Wraps everything in a SQLite transaction — either all succeeds or nothing changes.
    pub fn execute(conn: &Connection, input: &NewTransaction) -> Result<Transaction> {
        Self::validate(conn, input)?;

        conn.execute("BEGIN IMMEDIATE", [])?;

        let result = Self::execute_inner(conn, input);

        match result {
            Ok(txn) => {
                conn.execute("COMMIT", [])?;
                Ok(txn)
            }
            Err(e) => {
                let _ = conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }

    fn execute_inner(conn: &Connection, input: &NewTransaction) -> Result<Transaction> {
        match input.txn_type {
            TransactionType::Income    => Self::process_income(conn, input),
            TransactionType::Expense   => Self::process_expense(conn, input),
            TransactionType::Transfer  => Self::process_transfer(conn, input),
            TransactionType::InvestBuy => Self::process_invest_buy(conn, input),
            TransactionType::InvestSell => Self::process_invest_sell(conn, input),
        }
    }

    // ─── Income ───────────────────────────────────────────────────────────────

    fn process_income(conn: &Connection, input: &NewTransaction) -> Result<Transaction> {
        WalletRepo::add_balance(conn, &input.wallet_id, decimal_to_f64(input.amount))?;
        TransactionRepo::insert(conn, input)
    }

    // ─── Expense ─────────────────────────────────────────────────────────────

    fn process_expense(conn: &Connection, input: &NewTransaction) -> Result<Transaction> {
        Self::check_balance(conn, &input.wallet_id, input.amount)?;
        WalletRepo::add_balance(conn, &input.wallet_id, -decimal_to_f64(input.amount))?;
        TransactionRepo::insert(conn, input)
    }

    // ─── Transfer ────────────────────────────────────────────────────────────

    fn process_transfer(conn: &Connection, input: &NewTransaction) -> Result<Transaction> {
        let to_wallet_id = input.to_wallet_id.as_ref()
            .ok_or_else(|| AppError::Validation("Transfer requires to_wallet_id".into()))?;
        let to_amount = input.to_amount
            .ok_or_else(|| AppError::Validation("Transfer requires to_amount".into()))?;

        Self::check_balance(conn, &input.wallet_id, input.amount)?;
        WalletRepo::add_balance(conn, &input.wallet_id, -decimal_to_f64(input.amount))?;
        WalletRepo::add_balance(conn, to_wallet_id, decimal_to_f64(to_amount))?;
        TransactionRepo::insert(conn, input)
    }

    // ─── Invest Buy ──────────────────────────────────────────────────────────

    fn process_invest_buy(conn: &Connection, input: &NewTransaction) -> Result<Transaction> {
        let holding_id = input.holding_id.as_ref()
            .ok_or_else(|| AppError::Validation("invest_buy requires holding_id".into()))?;
        let qty = input.asset_quantity
            .ok_or_else(|| AppError::Validation("invest_buy requires asset_quantity".into()))?;
        let price = input.asset_price
            .ok_or_else(|| AppError::Validation("invest_buy requires asset_price".into()))?;

        // Deduct cost from wallet
        let total_cost = qty * price;
        Self::check_balance(conn, &input.wallet_id, total_cost)?;
        WalletRepo::add_balance(conn, &input.wallet_id, -decimal_to_f64(total_cost))?;

        // Update holding: quantity += qty, recalculate avg price
        HoldingService::apply_buy(conn, holding_id, qty, price)?;

        TransactionRepo::insert(conn, input)
    }

    // ─── Invest Sell ─────────────────────────────────────────────────────────

    fn process_invest_sell(conn: &Connection, input: &NewTransaction) -> Result<Transaction> {
        let holding_id = input.holding_id.as_ref()
            .ok_or_else(|| AppError::Validation("invest_sell requires holding_id".into()))?;
        let qty = input.asset_quantity
            .ok_or_else(|| AppError::Validation("invest_sell requires asset_quantity".into()))?;
        let price = input.asset_price
            .ok_or_else(|| AppError::Validation("invest_sell requires asset_price".into()))?;

        // Check sufficient holding quantity
        let current_qty = HoldingRepo::get_quantity(conn, holding_id)?;
        if qty > current_qty {
            return Err(AppError::InsufficientQuantity(holding_id.clone()));
        }

        // Reduce holding quantity
        HoldingService::apply_sell(conn, holding_id, qty)?;

        // Add proceeds to wallet
        let total_proceeds = qty * price;
        WalletRepo::add_balance(conn, &input.wallet_id, decimal_to_f64(total_proceeds))?;

        TransactionRepo::insert(conn, input)
    }

    // ─── Undo / Delete ───────────────────────────────────────────────────────

    /// Reverse a transaction — restores all balances/positions then deletes the record.
    pub fn delete(conn: &Connection, txn_id: &str) -> Result<()> {
        let txn = TransactionRepo::find_by_id(conn, txn_id)?
            .ok_or_else(|| AppError::NotFound(format!("transaction '{txn_id}'")))?;

        conn.execute("BEGIN IMMEDIATE", [])?;

        let result = Self::reverse_inner(conn, &txn);
        match result {
            Ok(_) => {
                conn.execute("COMMIT", [])?;
                Ok(())
            }
            Err(e) => {
                let _ = conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }

    fn reverse_inner(conn: &Connection, txn: &Transaction) -> Result<()> {
        match txn.txn_type {
            TransactionType::Income => {
                WalletRepo::add_balance(conn, &txn.wallet_id, -txn.amount_f64())?;
            }
            TransactionType::Expense => {
                WalletRepo::add_balance(conn, &txn.wallet_id, txn.amount_f64())?;
            }
            TransactionType::Transfer => {
                let to_id = txn.to_wallet_id.as_ref()
                    .ok_or_else(|| AppError::Validation("Corrupt transfer: missing to_wallet_id".into()))?;
                let to_amount = txn.to_amount
                    .ok_or_else(|| AppError::Validation("Corrupt transfer: missing to_amount".into()))?;
                WalletRepo::add_balance(conn, &txn.wallet_id, txn.amount_f64())?;
                WalletRepo::add_balance(conn, to_id, -decimal_to_f64(to_amount))?;
            }
            TransactionType::InvestBuy => {
                let holding_id = txn.holding_id.as_ref()
                    .ok_or_else(|| AppError::Validation("Corrupt invest_buy: missing holding_id".into()))?;
                let qty = txn.asset_quantity
                    .ok_or_else(|| AppError::Validation("Corrupt invest_buy: missing asset_quantity".into()))?;
                let total = txn.amount;
                WalletRepo::add_balance(conn, &txn.wallet_id, decimal_to_f64(total))?;
                // Reduce holding quantity; avg price recalculation is approximate on reversal
                let current_qty = HoldingRepo::get_quantity(conn, holding_id)?;
                let new_qty = (current_qty - qty).max(Decimal::ZERO);
                let avg = HoldingRepo::get_avg_price(conn, holding_id)?;
                HoldingRepo::update_position(conn, holding_id, new_qty, avg)?;
            }
            TransactionType::InvestSell => {
                let holding_id = txn.holding_id.as_ref()
                    .ok_or_else(|| AppError::Validation("Corrupt invest_sell: missing holding_id".into()))?;
                let qty = txn.asset_quantity
                    .ok_or_else(|| AppError::Validation("Corrupt invest_sell: missing asset_quantity".into()))?;
                let price = txn.asset_price
                    .ok_or_else(|| AppError::Validation("Corrupt invest_sell: missing asset_price".into()))?;
                let total = txn.amount;
                WalletRepo::add_balance(conn, &txn.wallet_id, -decimal_to_f64(total))?;
                // Restore holding quantity (avg price unchanged — best approximation)
                let current_qty = HoldingRepo::get_quantity(conn, holding_id)?;
                let avg = HoldingRepo::get_avg_price(conn, holding_id)?;
                // On reversal we also need to rebuild avg — using the old sell price as a proxy
                let new_qty = current_qty + qty;
                let new_avg = if new_qty > Decimal::ZERO {
                    (avg * current_qty + price * qty) / new_qty
                } else {
                    avg
                };
                HoldingRepo::update_position(conn, holding_id, new_qty, new_avg)?;
            }
        }

        TransactionRepo::delete(conn, &txn.id)?;
        Ok(())
    }

    // ─── Validation ──────────────────────────────────────────────────────────

    fn validate(conn: &Connection, input: &NewTransaction) -> Result<()> {
        if input.amount <= Decimal::ZERO {
            return Err(AppError::Validation("Amount must be greater than zero".into()));
        }

        // Wallet must exist
        WalletRepo::find_by_id(conn, &input.wallet_id)?
            .ok_or_else(|| AppError::NotFound(format!("wallet '{}'", input.wallet_id)))?;

        match input.txn_type {
            TransactionType::Income => {
                if input.income_type.is_none() {
                    return Err(AppError::Validation("income requires income_type".into()));
                }
            }
            TransactionType::Expense => {
                if input.category_id.is_none() {
                    return Err(AppError::Validation("expense requires category_id".into()));
                }
            }
            TransactionType::Transfer => {
                let to_id = input.to_wallet_id.as_ref()
                    .ok_or_else(|| AppError::Validation("transfer requires to_wallet_id".into()))?;
                if to_id == &input.wallet_id {
                    return Err(AppError::Validation("Cannot transfer to the same wallet".into()));
                }
                if input.to_amount.is_none() {
                    return Err(AppError::Validation("transfer requires to_amount".into()));
                }
            }
            TransactionType::InvestBuy | TransactionType::InvestSell => {
                if input.holding_id.is_none() {
                    return Err(AppError::Validation("invest transaction requires holding_id".into()));
                }
                if input.asset_quantity.map_or(true, |q| q <= Decimal::ZERO) {
                    return Err(AppError::Validation("asset_quantity must be > 0".into()));
                }
                if input.asset_price.map_or(true, |p| p <= Decimal::ZERO) {
                    return Err(AppError::Validation("asset_price must be > 0".into()));
                }
            }
        }

        Ok(())
    }

    // ─── Balance check ────────────────────────────────────────────────────────

    fn check_balance(conn: &Connection, wallet_id: &str, amount: Decimal) -> Result<()> {
        let balance_f64: f64 = conn.query_row(
            "SELECT balance FROM wallets WHERE id = ?1",
            [wallet_id],
            |row| row.get(0),
        ).map_err(|_| AppError::NotFound(format!("wallet '{wallet_id}'")))?;

        let balance = f64_to_decimal(balance_f64);
        if amount > balance {
            return Err(AppError::InsufficientBalance(wallet_id.to_string()));
        }
        Ok(())
    }
}
