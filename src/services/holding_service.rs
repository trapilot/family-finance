use rusqlite::Connection;
use rust_decimal::Decimal;

use crate::error::{AppError, Result};
use crate::models::holding::NewHolding;
use crate::repository::{HoldingRepo, WalletRepo};

pub struct HoldingService;

impl HoldingService {
    /// Get or create holding for wallet+symbol; returns holding_id
    pub fn get_or_create(conn: &Connection, wallet_id: &str, symbol: &str, name: Option<String>, asset_type: crate::models::AssetType) -> Result<String> {
        if let Some(h) = HoldingRepo::find_by_wallet_and_symbol(conn, wallet_id, symbol)? {
            return Ok(h.id);
        }

        // Verify wallet exists and is investment type
        let wallet = WalletRepo::find_by_id(conn, wallet_id)?
            .ok_or_else(|| AppError::NotFound(format!("wallet '{wallet_id}'")))?;

        if wallet.wallet_type != crate::models::WalletType::Investment {
            return Err(AppError::Validation(
                format!("Wallet '{}' is not an investment wallet", wallet.name),
            ));
        }

        let holding = HoldingRepo::create(conn, &NewHolding {
            wallet_id: wallet_id.to_string(),
            symbol: symbol.to_string(),
            name,
            asset_type,
        })?;

        Ok(holding.id)
    }

    /// Apply a buy: update quantity and recalculate weighted average price
    /// new_avg = (old_qty * old_avg + buy_qty * buy_price) / (old_qty + buy_qty)
    pub fn apply_buy(conn: &Connection, holding_id: &str, qty: Decimal, price: Decimal) -> Result<()> {
        let holding = HoldingRepo::find_by_id(conn, holding_id)?
            .ok_or_else(|| AppError::NotFound(format!("holding '{holding_id}'")))?;

        let old_qty = holding.quantity;
        let old_avg = holding.avg_buy_price;

        let new_qty = old_qty + qty;
        let new_avg = if new_qty > Decimal::ZERO {
            (old_qty * old_avg + qty * price) / new_qty
        } else {
            price
        };

        HoldingRepo::update_position(conn, holding_id, new_qty, new_avg)
    }

    /// Apply a sell: reduce quantity (avg price unchanged)
    pub fn apply_sell(conn: &Connection, holding_id: &str, qty: Decimal) -> Result<()> {
        let holding = HoldingRepo::find_by_id(conn, holding_id)?
            .ok_or_else(|| AppError::NotFound(format!("holding '{holding_id}'")))?;

        if qty > holding.quantity {
            return Err(AppError::InsufficientQuantity(holding_id.to_string()));
        }

        let new_qty = holding.quantity - qty;
        // avg_buy_price stays the same when selling (FIFO-equivalent for avg cost method)
        HoldingRepo::update_position(conn, holding_id, new_qty, holding.avg_buy_price)
    }

    /// Realized P&L for a sell trade
    /// pnl = (sell_price - avg_buy_price) * qty
    pub fn realized_pnl(conn: &Connection, holding_id: &str, sell_qty: Decimal, sell_price: Decimal) -> Result<Decimal> {
        let avg = HoldingRepo::get_avg_price(conn, holding_id)?;
        Ok((sell_price - avg) * sell_qty)
    }

    /// Update the last market price (manual entry)
    pub fn update_last_price(conn: &Connection, holding_id: &str, price: Decimal) -> Result<()> {
        if price < Decimal::ZERO {
            return Err(AppError::Validation("Price cannot be negative".into()));
        }
        HoldingRepo::update_last_price(conn, holding_id, price)
    }

    /// All holdings for a wallet with computed P&L
    pub fn portfolio_summary(conn: &Connection, wallet_id: &str) -> Result<Vec<HoldingWithPnl>> {
        let holdings = HoldingRepo::list_by_wallet(conn, wallet_id)?;
        Ok(holdings.into_iter().map(|h| {
            let unrealized_pnl = h.unrealized_pnl();
            let market_value   = h.market_value();
            let cost_basis     = h.cost_basis();
            HoldingWithPnl { holding: h, unrealized_pnl, market_value, cost_basis }
        }).collect())
    }

    /// Total unrealized P&L across all wallets
    pub fn total_unrealized_pnl(conn: &Connection) -> Result<Decimal> {
        let holdings = HoldingRepo::list_all_active(conn)?;
        Ok(holdings.iter().filter_map(|h| h.unrealized_pnl()).sum())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HoldingWithPnl {
    pub holding:        crate::models::Holding,
    pub unrealized_pnl: Option<Decimal>,
    pub market_value:   Option<Decimal>,
    pub cost_basis:     Decimal,
}

impl HoldingWithPnl {
    pub fn unrealized_pnl_pct(&self) -> Option<Decimal> {
        let pnl = self.unrealized_pnl?;
        if self.cost_basis == Decimal::ZERO {
            return None;
        }
        Some((pnl / self.cost_basis) * Decimal::from(100))
    }
}
