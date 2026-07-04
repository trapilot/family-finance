use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::app::AppState;
use crate::models::{Category, Family, Holding, Member, Transaction, Wallet};
use crate::repository::{CategoryRepo, FamilyRepo, HoldingRepo, TransactionRepo, WalletRepo};
use crate::views::pin::PinVerifyModal;

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
    pub families:     Vec<Family>,
}

// ─── BackupSettings view ─────────────────────────────────────────────────────

// ─── Action to execute after PIN verification ─────────────────────────────────
#[derive(Clone, PartialEq)]
enum PendingAction {
    Export,
    ImportFromFile(PathBuf),
    ImportFileFromPicker,
}

#[component]
pub fn BackupSettings() -> Element {
    let state   = use_context::<AppState>();
    let status  = use_signal(|| String::new());
    let is_ok   = use_signal(|| true);
    let backups = use_signal(|| Vec::<PathBuf>::new());

    // PIN verification modal state
    let mut show_pin_modal = use_signal(|| false);
    let mut pending_action = use_signal(|| None::<PendingAction>);

    // ── Load backups on mount and when needed ──
    let mut load_backups_effect = backups.clone();
    use_effect(move || {
        load_backups_effect.set(list_backup_files());
    });

    // ── Pending action handlers ──
    let handle_export = {
        let state  = state.clone();
        let mut status = status.clone();
        let mut is_ok  = is_ok.clone();
        let mut backups = backups.clone();
        move || {
            let result = state.with_conn(|conn| export_backup(conn));
            match result {
                Ok(json) => {
                    let save_result = save_to_backup_folder(&json);
                    match save_result {
                        Ok(path) => {
                            status.set(format!("✅ Exported to {}", path.display()));
                            is_ok.set(true);
                            backups.set(list_backup_files());
                        }
                        Err(e) => {
                            status.set(format!("❌ Save failed: {e}"));
                            is_ok.set(false);
                        }
                    }
                }
                Err(e) => {
                    status.set(format!("❌ Export failed: {e}"));
                    is_ok.set(false);
                }
            }
        }
    };

    let handle_import = {
        let state = state.clone();
        let mut status = status.clone();
        let mut is_ok  = is_ok.clone();
        let mut backups = backups.clone();
        move |file_path: PathBuf| {
            let result = read_backup_file(&file_path).and_then(|json| state.with_conn(|conn| import_backup(conn, &json)));
            match result {
                Ok(stats) => {
                    status.set(format!("✅ Imported from {}: {}", file_path.display(), stats));
                    is_ok.set(true);
                    backups.set(list_backup_files());
                }
                Err(e) => {
                    status.set(format!("❌ Import failed: {e}"));
                    is_ok.set(false);
                }
            }
        }
    };

    let handle_verified = {
        let mut show_pin_modal = show_pin_modal.clone();
        let mut pending_action = pending_action.clone();
        let mut handle_export = handle_export.clone();
        let mut handle_import = handle_import.clone();
        move |_| {
            if let Some(action) = pending_action.take() {
                match action {
                    PendingAction::Export => {
                        handle_export();
                    }
                    PendingAction::ImportFromFile(path) => {
                        handle_import(path);
                    }
                    PendingAction::ImportFileFromPicker => {
                        // We'll handle this in the JS eval part when we get there
                    }
                }
            }
            show_pin_modal.set(false);
        }
    };

    // ── Trigger PIN check then export ──
    let export = {
        let mut show_pin_modal = show_pin_modal.clone();
        let mut pending_action = pending_action.clone();
        move |_: MouseEvent| {
            pending_action.set(Some(PendingAction::Export));
            show_pin_modal.set(true);
        }
    };

    // ── Import from saved file ──
    let import = {
        let mut show_pin_modal = show_pin_modal.clone();
        let mut pending_action = pending_action.clone();
        move |file_path: PathBuf| {
            pending_action.set(Some(PendingAction::ImportFromFile(file_path)));
            show_pin_modal.set(true);
        }
    };

    // ── Share backup file ──
    let share = {
        let mut status = status.clone();
        move |file_path: PathBuf| {
            let result = read_backup_file(&file_path);
            match result {
                Ok(json) => {
                    let _ = share_backup(&json, &file_path);
                    status.set(format!("✅ Sharing backup {}", file_path.display()));
                }
                Err(e) => {
                    status.set(format!("❌ Share failed: {e}"));
                }
            }
        }
    };

    // ── Delete backup file ──
    let delete = {
        let mut status = status.clone();
        let mut is_ok  = is_ok.clone();
        let mut backups = backups.clone();
        move |file_path: PathBuf| {
            let result = std::fs::remove_file(&file_path);
            match result {
                Ok(_) => {
                    status.set(format!("✅ Deleted backup {}", file_path.display()));
                    is_ok.set(true);
                    backups.set(list_backup_files());
                }
                Err(e) => {
                    status.set(format!("❌ Delete failed: {e}"));
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
                    "Export all wallets, holdings, categories, transactions, members and families as a .bk file."
                }
                button {
                    style: "width:100%; padding:13px; background:#6366f1; color:#fff; border:none; border-radius:12px; font-size:15px; font-weight:700; cursor:pointer;",
                    onclick: export,
                    "📤 Export .bk"
                }
            }

            // ── Existing backups ──
            if !backups.read().is_empty() {
                div {
                    style: "background:#fff; border-radius:12px; padding:16px; box-shadow:0 1px 3px rgba(0,0,0,.06);",
                    h2 { style: "margin:0 0 8px; font-size:16px; font-weight:700; color:#1f2937;", "Saved Backups" }
                    div { style: "display:flex; flex-direction:column; gap:8px;",
                        for path in backups.read().iter() {
                            BackupFileItem {
                                path: path.clone(),
                                on_import: import.clone(),
                                on_share: share.clone(),
                                on_delete: delete.clone(),
                            }
                        }
                    }
                }
            }

            // ── Import from file ──
            div {
                style: "background:#fff; border-radius:12px; padding:16px; box-shadow:0 1px 3px rgba(0,0,0,.06);",
                h2 { style: "margin:0 0 8px; font-size:16px; font-weight:700; color:#1f2937;", "Import from shared file" }
                p  { style: "margin:0 0 12px; font-size:13px; color:#ef4444; line-height:1.5; background:#fef2f2; border-radius:8px; padding:10px;",
                    "⚠️ Import will REPLACE all existing data. This cannot be undone. Select a shared backup file to import it."
                }
                label {
                    style: "display:block; width:100%; padding:13px; background:#6366f1; color:#fff; border:none; border-radius:12px; font-size:15px; font-weight:700; cursor:pointer; text-align:center;",
                    "📥 Import File"
                    input {
                        r#type: "file",
                        accept: ".bk,.ffbackup",
                        style: "display:none;",
                        oninput: move |_e| {
                            let state = state.clone();
                            let status = status.clone();
                            let is_ok = is_ok.clone();
                            let backups = backups.clone();
                            let js = r#"
                                (function() {
                                    const input = document.querySelector('input[type="file"][accept=".bk,.ffbackup"]');
                                    if (input && input.files && input.files.length > 0) {
                                        const file = input.files[0];
                                        const reader = new FileReader();
                                        reader.onload = function(e) {
                                            const content = e.target.result;
                                            const filename = file.name;
                                            // Return both
                                            dioxus.send(JSON.stringify({ filename, content }));
                                        };
                                        reader.onerror = function() {
                                            dioxus.send("error");
                                        };
                                        reader.readAsText(file);
                                    }
                                })();
                            "#;
                            let mut eval = dioxus::document::eval(js);
                            let mut status_clone = status.clone();
                            let mut is_ok_clone = is_ok.clone();
                            let mut backups_clone = backups.clone();
                            let state_clone = state.clone();
                            let _ = dioxus::prelude::spawn(async move {
                                if let Ok(result) = eval.recv::<String>().await {
                                    if result == "error" {
                                        status_clone.set("❌ Failed to read file".to_string());
                                        is_ok_clone.set(false);
                                        return;
                                    }
                                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&result) {
                                        let filename = data["filename"].as_str().unwrap_or("imported.bk").to_string();
                                        let content = data["content"].as_str().unwrap_or("").to_string();
                                        let save_result = save_to_backup_folder_from_string(&content, &filename);
                                        match save_result {
                                            Ok(path) => {
                                                let import_result = state_clone.with_conn(|conn| import_backup(conn, &content));
                                                match import_result {
                                                    Ok(stats) => {
                                                        status_clone.set(format!("✅ Imported from file {}: {}", path.display(), stats));
                                                        is_ok_clone.set(true);
                                                        backups_clone.set(list_backup_files());
                                                    }
                                                    Err(e) => {
                                                        status_clone.set(format!("❌ Import failed: {e}"));
                                                        is_ok_clone.set(false);
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                status_clone.set(format!("❌ Save failed: {e}"));
                                                is_ok_clone.set(false);
                                            }
                                        }
                                    }
                                }
                            });
                        },
                    }
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

            // ── PIN Verify Modal ──
            PinVerifyModal {
                is_visible: show_pin_modal,
                on_cancel: move |_| {
                    pending_action.set(None);
                    show_pin_modal.set(false);
                },
                on_verify: handle_verified,
            }
        }
    }
}

#[component]
fn BackupFileItem(
    path: PathBuf,
    on_import: EventHandler<PathBuf>,
    on_share: EventHandler<PathBuf>,
    on_delete: EventHandler<PathBuf>,
) -> Element {
    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
    let path_clone_import = path.clone();
    let path_clone_share = path.clone();
    let path_clone_delete = path.clone();

    rsx! {
        div {
            style: "display:flex; align-items:center; justify-content:space-between; padding:10px 12px; background:#f9fafb; border-radius:8px;",
            span {
                style: "font-size:13px; color:#374151;",
                "{filename}"
            }
            div {
                style: "display:flex; gap:8px;",
                button {
                    style: "padding:6px 10px; border:none; border-radius:6px; background:#ef4444; color:#fff; font-size:12px; font-weight:600; cursor:pointer;",
                    onclick: move |_| on_import.call(path_clone_import.clone()),
                    "Revert"
                }
                button {
                    style: "padding:6px 10px; border:none; border-radius:6px; background:#6366f1; color:#fff; font-size:12px; font-weight:600; cursor:pointer;",
                    onclick: move |_| on_share.call(path_clone_share.clone()),
                    "Share"
                }
                button {
                    style: "padding:6px 10px; border:none; border-radius:6px; background:#6b7280; color:#fff; font-size:12px; font-weight:600; cursor:pointer;",
                    onclick: move |_| on_delete.call(path_clone_delete.clone()),
                    "Delete"
                }
            }
        }
    }
}

// ─── Danger zone (reset data) ─────────────────────────────────────────────────

#[component]
fn DangerZone() -> Element {
    let state   = use_context::<AppState>();
    let mut show_pin_modal = use_signal(|| false);
    let mut confirmed = use_signal(|| false);

    let handle_delete = {
        let state = state.clone();
        move || {
            state.with_conn(|conn| {
                let _ = conn.execute_batch("
                    DELETE FROM transactions;
                    DELETE FROM holdings;
                    DELETE FROM members;
                    DELETE FROM families;
                    DELETE FROM wallets;
                    DELETE FROM categories WHERE is_system = 0;
                ");
            });
        }
    };

    let handle_verified = {
        let mut show_pin_modal = show_pin_modal.clone();
        let handle_delete = handle_delete.clone();
        move |_| {
            handle_delete();
            show_pin_modal.set(false);
        }
    };

    rsx! {
        div {
            style: "background:#fff; border-radius:12px; padding:16px; box-shadow:0 1px 3px rgba(0,0,0,.06); border:1px solid #fee2e2;",
            h2 { style: "margin:0 0 8px; font-size:16px; font-weight:700; color:#ef4444;", "⚠️ Danger Zone" }
            p  { style: "margin:0 0 12px; font-size:13px; color:#6b7280;", "Permanently delete all data. Cannot be undone." }

            button {
                style: "width:100%; padding:12px; background:transparent; color:#ef4444; border:1px solid #ef4444; border-radius:10px; font-size:14px; font-weight:600; cursor:pointer;",
                onclick: move |_| {
                    confirmed.set(true);
                    show_pin_modal.set(true);
                },
                "Delete All Data"
            }

            PinVerifyModal {
                is_visible: show_pin_modal,
                on_cancel: move |_| {
                    confirmed.set(false);
                    show_pin_modal.set(false);
                },
                on_verify: handle_verified,
            }
        }
    }
}

// ─── Backup logic ─────────────────────────────────────────────────────────────

fn export_backup(conn: &rusqlite::Connection) -> crate::error::Result<String> {
    let wallets      = WalletRepo::list_active(conn)?;
    let holdings     = HoldingRepo::list_all_active(conn)?;
    let categories   = CategoryRepo::list(conn)?;
    let transactions = TransactionRepo::list_all(conn)?;
    let members      = load_members(conn);
    let families     = FamilyRepo::list(conn)?;

    let data = BackupData {
        app_version:  "1.0.0".into(),
        exported_at:  crate::views::now_ts(),
        wallets,
        holdings,
        categories,
        transactions,
        members,
        families,
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
            DELETE FROM families;
        ")?;

        // Insert families first because members have foreign key
        for f in &data.families {
            conn.execute(
                "INSERT OR REPLACE INTO families
                 (id,name,common_address,note,created_at)
                 VALUES (?1,?2,?3,?4,?5)",
                rusqlite::params![
                    f.id, f.name, f.common_address, f.note, f.created_at,
                ],
            )?;
        }

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

        // Insert members (now includes family_id)
        for m in &data.members {
            conn.execute(
                "INSERT OR REPLACE INTO members
                 (id,family_id,full_name,birth_date,gender,phone,id_number,id_issue_date,id_issue_place,address,role,avatar_emoji,note,created_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
                rusqlite::params![
                    m.id, m.family_id, m.full_name, m.birth_date, m.gender.as_ref().map(|g| g.to_string()).unwrap_or_default(), m.phone,
                    m.id_number, m.id_issue_date, m.id_issue_place, m.address, m.role.to_string(), m.avatar_emoji,
                    m.note, m.created_at,
                ],
            )?;
        }

        Ok(())
    })();

    match result {
        Ok(_) => {
            conn.execute_batch("COMMIT")?;
            Ok(format!(
                "{} families, {} wallets, {} holdings, {} categories, {} transactions, {} members",
                data.families.len(), data.wallets.len(), data.holdings.len(),
                data.categories.len(), data.transactions.len(), data.members.len(),
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
        "SELECT id,family_id,full_name,birth_date,gender,phone,id_number,id_issue_date,id_issue_place,address,role,avatar_emoji,note,created_at FROM members"
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    stmt.query_map([], Member::from_row)
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default()
}

fn ensure_backup_folder() -> std::io::Result<PathBuf> {
    let backup_dir = PathBuf::from("backup");
    if !backup_dir.exists() {
        std::fs::create_dir_all(&backup_dir)?;
    }
    Ok(backup_dir)
}

fn list_backup_files() -> Vec<PathBuf> {
    let backup_dir = match ensure_backup_folder() {
        Ok(d) => d,
        Err(_) => return vec![],
    };

    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&backup_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("bk") {
                files.push(path);
            }
        }
    }

    // Sort by modified time descending
    files.sort_by(|a, b| {
        let a_time = a.metadata().and_then(|m| m.modified()).ok();
        let b_time = b.metadata().and_then(|m| m.modified()).ok();
        b_time.cmp(&a_time)
    });

    files
}

fn save_to_backup_folder(json: &str) -> crate::error::Result<PathBuf> {
    use std::io::Write;
    use chrono::{Local, TimeZone};
    let backup_dir = ensure_backup_folder()?;
    let ts = crate::views::now_ts();
    let datetime = Local.timestamp_opt(ts, 0).single().unwrap_or_else(|| Local::now());
    let filename = format!("{}.bk", datetime.format("%Y_%m_%d_%H_%M_%S"));
    let path = backup_dir.join(filename);
    let mut f = std::fs::File::create(&path)?;
    f.write_all(json.as_bytes())?;
    Ok(path)
}

fn save_to_backup_folder_from_string(json: &str, original_filename: &str) -> crate::error::Result<PathBuf> {
    use std::io::Write;
    let backup_dir = ensure_backup_folder()?;
    let mut path = backup_dir.join(original_filename);
    let mut counter = 1;
    while path.exists() {
        let stem = std::path::Path::new(original_filename).file_stem().and_then(|s| s.to_str()).unwrap_or("imported");
        let ext = std::path::Path::new(original_filename).extension().and_then(|s| s.to_str()).unwrap_or("bk");
        path = backup_dir.join(format!("{}_{}.{}", stem, counter, ext));
        counter += 1;
    }
    let mut f = std::fs::File::create(&path)?;
    f.write_all(json.as_bytes())?;
    Ok(path)
}

fn read_backup_file(path: &PathBuf) -> crate::error::Result<String> {
    Ok(std::fs::read_to_string(path)?)
}

fn share_backup(json: &str, path: &PathBuf) -> crate::error::Result<()> {
    // On web/desktop: try to download or copy to clipboard
    // On mobile: use native share sheet
    // For now, we'll just copy to clipboard and log
    let js = format!(r#"
        async function shareBackup() {{
            try {{
                // Try to copy to clipboard first
                await navigator.clipboard.writeText('{}');
                // Try to use Web Share API if available
                if (navigator.share) {{
                    const file = new File([`{}`], '{}', {{type: 'application/json'}});
                    await navigator.share({{
                        title: 'Family Finance Backup',
                        text: 'Family Finance Backup File',
                        files: [file]
                    }});
                }}
            }} catch (e) {{
                console.log('Sharing failed:', e);
            }}
        }}
        shareBackup();
    "#, json.replace('\\', "\\\\").replace('\'', "\\'"), json.replace('\\', "\\\\").replace('\'', "\\'"), path.file_name().unwrap_or_default().to_str().unwrap_or_default());
    
    let _ = dioxus::document::eval(&js);
    Ok(())
}
