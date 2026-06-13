use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::models::{Category, Holding, Member, Transaction, Wallet};
use crate::repository::{CategoryRepo, HoldingRepo, TransactionRepo, WalletRepo};

// ─── Backup format ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupData {
    pub app_version:  String,
    pub exported_at:  i64,
    pub wallets:      Vec<Wallet>,
    pub holdings:     Vec<Holding>,
    pub categories:   Vec<Category>,
    pub transactions: Vec<Transaction>,
    pub members:      Vec<Member>,
}

// ─── BackupSettings view ─────────────────────────────────────────────────────

#[component]
pub fn BackupSettings() -> Element {
    let state   = use_context::<AppState>();
    let status  = use_signal(|| String::new());
    let is_ok   = use_signal(|| true);

    // ── Export ──
    let export = {
        let state  = state.clone();
        let mut status = status.clone();
        let mut is_ok  = is_ok.clone();
        move |_: MouseEvent| {
            let result = state.with_conn(|conn| export_backup(conn));
            match result {
                Ok(json) => {
                    // On native mobile this would use the share sheet.
                    // For now: copy to clipboard via JS interop or log.
                    let _ = save_to_downloads(&json);
                    status.set(format!("✅ Exported — {} bytes", json.len()));
                    is_ok.set(true);
                }
                Err(e) => {
                    status.set(format!("❌ Export failed: {e}"));
                    is_ok.set(false);
                }
            }
        }
    };

    // ── Import (paste JSON) ──
    let mut import_text = use_signal(|| String::new());
    let import = {
        let state = state.clone();
        let mut import_text = import_text.clone();
        let mut status = status.clone();
        let mut is_ok  = is_ok.clone();
        move |_: MouseEvent| {
            let json = import_text.read().clone();
            if json.trim().is_empty() {
                status.set("❌ Paste .ffbackup JSON first".into());
                is_ok.set(false);
                return;
            }
            let result = state.with_conn(|conn| import_backup(conn, &json));
            match result {
                Ok(stats) => {
                    status.set(format!("✅ Imported: {}", stats));
                    is_ok.set(true);
                    import_text.set(String::new());
                }
                Err(e) => {
                    status.set(format!("❌ Import failed: {e}"));
                    is_ok.set(false);
                }
            }
        }
    };

    rsx! {
        div {
            style: "display:flex; flex-direction:column; gap:16px;",

            // ── App info ──
            div {
                style: "background:#fff; border-radius:12px; padding:16px; box-shadow:0 1px 3px rgba(0,0,0,.06);",
                div { style: "font-size:11px; color:#9ca3af; margin-bottom:4px;", "APP" }
                div { style: "font-size:16px; font-weight:700; color:#1f2937;", "Family Finance" }
                div { style: "font-size:13px; color:#6b7280; margin-top:2px;", "Version 1.0.0 · SQLite local storage" }
            }

            // ── Export ──
            div {
                style: "background:#fff; border-radius:12px; padding:16px; box-shadow:0 1px 3px rgba(0,0,0,.06);",
                h2 { style: "margin:0 0 8px; font-size:16px; font-weight:700; color:#1f2937;", "Export Backup" }
                p  { style: "margin:0 0 12px; font-size:13px; color:#6b7280; line-height:1.5;",
                    "Export all wallets, holdings, categories and transactions as a .ffbackup file."
                }
                button {
                    style: "width:100%; padding:13px; background:#6366f1; color:#fff; border:none; border-radius:12px; font-size:15px; font-weight:700; cursor:pointer;",
                    onclick: export,
                    "📤 Export .ffbackup"
                }
            }

            // ── Import ──
            div {
                style: "background:#fff; border-radius:12px; padding:16px; box-shadow:0 1px 3px rgba(0,0,0,.06);",
                h2 { style: "margin:0 0 8px; font-size:16px; font-weight:700; color:#1f2937;", "Import Backup" }
                p  { style: "margin:0 0 12px; font-size:13px; color:#ef4444; line-height:1.5; background:#fef2f2; border-radius:8px; padding:10px;",
                    "⚠️ Import will REPLACE all existing data. This cannot be undone."
                }
                textarea {
                    style: "width:100%; height:120px; border:1px solid #e5e7eb; border-radius:8px; padding:10px; font-size:12px; font-family:monospace; resize:vertical; outline:none; box-sizing:border-box;",
                    placeholder: "Paste .ffbackup JSON here…",
                    value: "{import_text.read()}",
                    oninput: move |e| { import_text.set(e.value()); },
                }
                button {
                    style: "width:100%; padding:13px; background:#ef4444; color:#fff; border:none; border-radius:12px; font-size:15px; font-weight:700; cursor:pointer; margin-top:8px;",
                    onclick: import,
                    "📥 Import & Replace Data"
                }
            }

            // ── Status ──
            if !status.read().is_empty() {
                div {
                    style: if *is_ok.read() {
                        "background:#f0fdf4; border-radius:8px; padding:12px; font-size:14px; color:#15803d;"
                    } else {
                        "background:#fef2f2; border-radius:8px; padding:12px; font-size:14px; color:#dc2626;"
                    },
                    "{status.read()}"
                }
            }

            // ── Danger zone ──
            DangerZone {}
        }
    }
}

// ─── Danger zone (reset data) ─────────────────────────────────────────────────

#[component]
fn DangerZone() -> Element {
    let state   = use_context::<AppState>();
    let mut confirm = use_signal(|| false);

    rsx! {
        div {
            style: "background:#fff; border-radius:12px; padding:16px; box-shadow:0 1px 3px rgba(0,0,0,.06); border:1px solid #fee2e2;",
            h2 { style: "margin:0 0 8px; font-size:16px; font-weight:700; color:#ef4444;", "⚠️ Danger Zone" }
            p  { style: "margin:0 0 12px; font-size:13px; color:#6b7280;", "Permanently delete all data. Cannot be undone." }

            button {
                style: "width:100%; padding:12px; background:transparent; color:#ef4444; border:1px solid #ef4444; border-radius:10px; font-size:14px; font-weight:600; cursor:pointer;",
                onclick: move |_| { confirm.set(true); },
                "Delete All Data"
            }

            if *confirm.read() {
                div {
                    style: "position:fixed; inset:0; background:rgba(0,0,0,.5); display:flex; align-items:center; justify-content:center; z-index:200;",
                    div {
                        style: "background:#fff; border-radius:16px; padding:24px; margin:16px; max-width:320px; width:100%;",
                        h3 { style: "margin:0 0 8px; font-size:18px; color:#ef4444;", "Are you sure?" }
                        p  { style: "margin:0 0 20px; font-size:14px; color:#6b7280;", "All wallets, transactions, and holdings will be permanently deleted." }
                        div { style: "display:flex; gap:10px;",
                            button {
                                style: "flex:1; padding:11px; border:1px solid #e5e7eb; border-radius:8px; background:#fff; cursor:pointer; font-size:14px;",
                                onclick: move |_| { confirm.set(false); },
                                "Cancel"
                            }
                            button {
                                style: "flex:1; padding:11px; border:none; border-radius:8px; background:#ef4444; color:#fff; cursor:pointer; font-size:14px; font-weight:700;",
                                onclick: {
                                    let state = state.clone();
                                    move |_| {
                                        state.with_conn(|conn| {
                                            let _ = conn.execute_batch("
                                                DELETE FROM transactions;
                                                DELETE FROM holdings;
                                                DELETE FROM wallets;
                                                DELETE FROM categories WHERE is_system = 0;
                                            ");
                                        });
                                        confirm.set(false);
                                    }
                                },
                                "Yes, Delete All"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ─── Backup logic ─────────────────────────────────────────────────────────────

fn export_backup(conn: &rusqlite::Connection) -> crate::error::Result<String> {
    // use crate::repository::transaction_repo::TxnFilter;

    let wallets      = WalletRepo::list_active(conn)?;
    let holdings     = HoldingRepo::list_all_active(conn)?;
    let categories   = CategoryRepo::list(conn)?;
    let transactions = TransactionRepo::list_all(conn)?;

    // Members — Phase 2 (table exists but may be empty)
    let members = load_members(conn);

    let data = BackupData {
        app_version:  "1.0.0".into(),
        exported_at:  crate::views::now_ts(),
        wallets,
        holdings,
        categories,
        transactions,
        members,
    };

    Ok(serde_json::to_string_pretty(&data)?)
}

fn import_backup(conn: &rusqlite::Connection, json: &str) -> crate::error::Result<String> {
    let data: BackupData = serde_json::from_str(json)?;

    conn.execute_batch("BEGIN IMMEDIATE")?;

    let result = (|| -> crate::error::Result<()> {
        // Clear existing data
        conn.execute_batch("
            DELETE FROM transactions;
            DELETE FROM holdings;
            DELETE FROM wallets;
            DELETE FROM categories WHERE is_system = 0;
            DELETE FROM members;
        ")?;

        // Insert wallets
        for w in &data.wallets {
            conn.execute(
                "INSERT OR REPLACE INTO wallets
                 (id,name,wallet_type,currency,balance,broker,icon,color,is_active,sort_order,created_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                rusqlite::params![
                    w.id, w.name, w.wallet_type.to_string(), w.currency.to_string(),
                    w.balance_f64(), w.broker, w.icon, w.color,
                    w.is_active as i32, w.sort_order, w.created_at,
                ],
            )?;
        }

        // Insert holdings
        for h in &data.holdings {
            conn.execute(
                "INSERT OR REPLACE INTO holdings
                 (id,wallet_id,symbol,name,asset_type,quantity,avg_buy_price,last_price,last_price_at,created_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                rusqlite::params![
                    h.id, h.wallet_id, h.symbol, h.name, h.asset_type.to_string(),
                    h.quantity_f64(), h.avg_buy_price_f64(),
                    h.last_price.map(|p| crate::models::decimal_to_f64(p)),
                    h.last_price_at, h.created_at,
                ],
            )?;
        }

        // Insert categories (skip system — already seeded)
        for c in data.categories.iter().filter(|c| !c.is_system) {
            conn.execute(
                "INSERT OR REPLACE INTO categories
                 (id,name,icon,color,budget_amount,parent_id,sort_order,is_system,created_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,0,?8)",
                rusqlite::params![
                    c.id, c.name, c.icon, c.color,
                    c.budget_f64(), c.parent_id, c.sort_order, c.created_at,
                ],
            )?;
        }

        // Insert transactions (raw — no balance recalculation)
        for t in &data.transactions {
            conn.execute(
                "INSERT OR REPLACE INTO transactions
                 (id,txn_type,wallet_id,amount,currency,income_type,category_id,
                  to_wallet_id,to_amount,to_currency,holding_id,asset_quantity,asset_price,
                  note,txn_date,created_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
                rusqlite::params![
                    t.id, t.txn_type.to_string(), t.wallet_id, t.amount_f64(),
                    t.currency.to_string(),
                    t.income_type.as_ref().map(|i| i.to_string()),
                    t.category_id,
                    t.to_wallet_id,
                    t.to_amount.map(crate::models::decimal_to_f64),
                    t.to_currency.as_ref().map(|c| c.to_string()),
                    t.holding_id,
                    t.asset_quantity.map(crate::models::decimal_to_f64),
                    t.asset_price.map(crate::models::decimal_to_f64),
                    t.note, t.txn_date, t.created_at,
                ],
            )?;
        }

        Ok(())
    })();

    match result {
        Ok(_) => {
            conn.execute_batch("COMMIT")?;
            Ok(format!(
                "{} wallets, {} holdings, {} categories, {} transactions",
                data.wallets.len(), data.holdings.len(),
                data.categories.len(), data.transactions.len()
            ))
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

fn load_members(conn: &rusqlite::Connection) -> Vec<Member> {
    use crate::models::Member;
    let mut stmt = match conn.prepare(
        "SELECT id,full_name,birth_date,gender,phone,id_number,id_issue_date,id_issue_place,address,role,note,created_at FROM members"
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    stmt.query_map([], Member::from_row)
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default()
}

/// Save JSON to the downloads/documents directory.
/// On iOS this would use the native share sheet via Dioxus mobile API.
/// Stub for now — replace with actual platform API call.
fn save_to_downloads(json: &str) -> std::io::Result<()> {
    use std::io::Write;
    let ts = crate::views::now_ts();
    let filename = format!("family_finance_{ts}.ffbackup");

    // On device: use dioxus mobile file picker / share sheet
    // For desktop dev: write to current dir
    let path = std::path::PathBuf::from(&filename);
    let mut f = std::fs::File::create(&path)?;
    f.write_all(json.as_bytes())?;
    Ok(())
}
