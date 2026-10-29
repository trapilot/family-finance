use dioxus::prelude::*;
use crate::i18n::use_i18n;

use crate::app::AppState;
use crate::models::{Transaction, TransactionType};
use crate::repository::transaction_repo::TxnFilter;
use crate::repository::{CategoryRepo, TransactionRepo};
use crate::services::TransactionService;
use crate::views::{fmt_currency, ts_to_date_str};
use std::collections::HashMap;
use super::TxnView;

#[component]
pub fn TransactionList(view: Signal<TxnView>) -> Element {
    let t = use_i18n();
    let state = use_context::<AppState>();

    let mut txns        = use_signal(|| vec![]);
    // Types shown; toggled by the chips (at least one stays selected)
    let mut filter_types = use_signal(|| vec![
        TransactionType::Income,
        TransactionType::Expense,
        TransactionType::Transfer,
    ]);
    let mut search_note = use_signal(|| String::new());
    let mut show_confirm_delete = use_signal(|| None::<String>); // holds txn id
    // category id → (icon, name) for the rows
    let categories = use_signal({
        let state = state.clone();
        move || -> HashMap<String, (String, String)> {
            state.with_conn(|conn| CategoryRepo::list(conn).unwrap_or_default())
                .into_iter()
                .map(|c| (c.id, (c.icon.unwrap_or_default(), c.name)))
                .collect()
        }
    });

    // ── Load ──
    let mut txns_effect = txns.clone();
    let state_effect = state.clone();

    use_effect(move || {
        let types = filter_types.read().clone();

        state_effect.with_conn(|conn| {
            if let Ok(list) = TransactionRepo::list(conn, &TxnFilter {
                txn_types: types,
                limit: Some(100),
                ..Default::default()
            }) {
                txns_effect.set(list);
            }
        });
    });

    // Search filter in memory
    let query = search_note.read().to_lowercase();
    let visible: Vec<Transaction> = txns.read().iter()
        .filter(|t| {
            if query.is_empty() { return true; }
            t.note.as_deref().unwrap_or("").to_lowercase().contains(&query)
        })
        .cloned()
        .collect();

    rsx! {
        div {
            style: "padding:16px; display:flex; flex-direction:column; gap:12px;",

            // ── Header ──
            div {
                style: "display:flex; justify-content:space-between; align-items:center;",
                h1 { style: "margin:0; font-size:20px; font-weight:700; color:#1f2937;", {t.t("Transactions")} }
                button {
                    style: "background:#6366f1; color:#fff; border:none; border-radius:20px; padding:8px 16px; font-size:14px; font-weight:600; cursor:pointer;",
                    onclick: move |_| { view.set(TxnView::Add); },
                    {t.t("+ Add")}
                }
            }

            // ── Search ──
            input {
                style: "border:1px solid #e5e7eb; border-radius:8px; padding:10px 12px; font-size:14px; outline:none; background:#fff;",
                r#type: "text",
                placeholder: t.t("Search by note…"),
                value: "{search_note.read()}",
                oninput: move |e| {
                    search_note.set(e.value());
                },
            }

            // ── Type Filter (multi-select) ──
            div {
                style: "display:flex; gap:6px; overflow-x:auto; padding-bottom:4px;",
                for (tt, label) in [
                    (TransactionType::Income,    t.t("Income")),
                    (TransactionType::Expense,   t.t("Expense")),
                    (TransactionType::Transfer,  t.t("Transfer")),
                    (TransactionType::InvestBuy, t.t("Buy")),
                    (TransactionType::InvestSell,t.t("Sell")),
                    (TransactionType::OpeningBalance, t.t("Opening Balance")),
                ] {
                    {
                        let tt_clone = tt.clone();
                        rsx! {
                            FilterChip {
                                label,
                                active: filter_types.read().contains(&tt),
                                onclick: move |_| {
                                    let mut types = filter_types.write();
                                    if let Some(pos) = types.iter().position(|x| *x == tt_clone) {
                                        // Keep at least one type selected
                                        if types.len() > 1 { types.remove(pos); }
                                    } else {
                                        types.push(tt_clone.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ── List ──
            if visible.is_empty() {
                div {
                    style: "text-align:center; padding:40px 0; color:#9ca3af; font-size:14px;",
                    {t.t("No transactions found")}
                }
            }

            for txn in visible.iter() {
                TxnRow {
                    category: txn.category_id.as_deref()
                        .and_then(|id| categories.read().get(id).cloned())
                        .map(|(icon, name)| format!("{icon} {}", t.tr(&name)).trim().to_string()),
                    txn: txn.clone(),
                    on_edit: {
                        let txn = txn.clone();
                        move |_| { view.set(TxnView::Edit(txn.clone())); }
                    },
                    on_delete: {
                        let id = txn.id.clone();
                        move |_| { show_confirm_delete.set(Some(id.clone())); }
                    }
                }
            }

            // ── Delete confirm modal ──
            if let Some(txn_id) = show_confirm_delete.read().clone() {
                div {
                    style: "position:fixed; inset:0; background:rgba(0,0,0,.4); display:flex; align-items:center; justify-content:center; z-index:100;",
                    div {
                        style: "background:#fff; border-radius:16px; padding:24px; margin:16px; max-width:320px; width:100%;",
                        h3 { style: "margin:0 0 8px; font-size:17px;", {t.t("Delete transaction?")} }
                        p  { style: "margin:0 0 20px; font-size:14px; color:#6b7280;", {t.t("This will reverse all balance changes.")} }
                        div {
                            style: "display:flex; gap:10px;",
                            button {
                                style: "flex:1; padding:10px; border:1px solid #e5e7eb; border-radius:8px; background:#fff; cursor:pointer; font-size:14px;",
                                onclick: move |_| { show_confirm_delete.set(None); },
                                {t.t("Cancel")}
                            }
                            button {
                                style: "flex:1; padding:10px; border:none; border-radius:8px; background:#ef4444; color:#fff; cursor:pointer; font-size:14px; font-weight:600;",
                                onclick: {
                                    let state = state.clone();
                                    let txn_id = txn_id.clone();
                                    move |_| {
                                        state.with_conn(|conn| {
                                            let _ = TransactionService::delete(conn, &txn_id);
                                        });
                                        show_confirm_delete.set(None);
                                        
                                        let types = filter_types.read().clone();

                                        state.with_conn(|conn| {
                                            if let Ok(list) = TransactionRepo::list(conn, &TxnFilter {
                                                txn_types: types,
                                                limit: Some(100),
                                                ..Default::default()
                                            }) {
                                                txns.set(list);
                                            }
                                        });
                                        
                                    }
                                },
                                {t.t("Delete")}
                            }
                        }
                    }
                }
            }
        }
    }
}

// ─── TxnRow ───────────────────────────────────────────────────────────────────

#[component]
fn TxnRow(
    txn: Transaction,
    /// "🍜 Food & Drinks" for expenses
    category: Option<String>,
    on_edit: EventHandler<MouseEvent>,
    on_delete: EventHandler<MouseEvent>,
) -> Element {
    let t = use_i18n();
    let (icon, amount_color) = match txn.txn_type {
        TransactionType::Income     => ("↑", "#10b981"),
        TransactionType::Expense    => ("↓", "#ef4444"),
        TransactionType::Transfer   => ("⇄", "#6366f1"),
        TransactionType::InvestBuy  => ("📈", "#f59e0b"),
        TransactionType::InvestSell => ("📉", "#8b5cf6"),
        TransactionType::OpeningBalance => ("🏁", "#0ea5e9"),
    };
    let amount_str = fmt_currency(txn.amount, &txn.currency);
    let date_str   = ts_to_date_str(txn.txn_date);
    let type_label = t.code(&txn.txn_type.to_string());
    let note       = txn.note.clone().unwrap_or_default();

    rsx! {
        div {
            style: "background:#fff; border-radius:12px; padding:12px; box-shadow:0 1px 3px rgba(0,0,0,.06); display:flex; align-items:center; gap:12px;",

            span {
                style: "width:36px; height:36px; border-radius:50%; background:#f3f4f6; display:flex; align-items:center; justify-content:center; font-size:18px; flex-shrink:0;",
                "{icon}"
            }

            div { style: "flex:1; min-width:0;",
                div { style: "font-size:13px; font-weight:600; color:#374151; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;",
                    span { style: "text-transform:capitalize;", "{type_label}" }
                    if let Some(category) = category {
                        span { style: "font-weight:500; color:#6b7280;", " · {category}" }
                    }
                }
                if !note.is_empty() {
                    div { style: "font-size:12px; color:#6b7280; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;", "{note}" }
                }
                div { style: "font-size:11px; color:#9ca3af;", "{date_str}" }
            }

            div { style: "text-align:right; flex-shrink:0;",
                div { style: "font-size:14px; font-weight:700; color:{amount_color};", "{amount_str}" }
            }

            div { style: "display:flex; flex-direction:column; gap:4px; flex-shrink:0;",
                button {
                    style: "border:none; background:transparent; cursor:pointer; font-size:16px; padding:2px;",
                    onclick: move |e| { on_edit.call(e); },
                    "✏️"
                }
                button {
                    style: "border:none; background:transparent; cursor:pointer; font-size:16px; padding:2px;",
                    onclick: move |e| { on_delete.call(e); },
                    "🗑"
                }
            }
        }
    }
}

// ─── FilterChip ───────────────────────────────────────────────────────────────

#[component]
fn FilterChip(label: &'static str, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            style: if active {
                "border:none; border-radius:20px; padding:6px 14px; font-size:12px; font-weight:600; cursor:pointer; white-space:nowrap; background:#6366f1; color:#fff;"
            } else {
                "border:1px solid #e5e7eb; border-radius:20px; padding:6px 14px; font-size:12px; cursor:pointer; white-space:nowrap; background:#fff; color:#6b7280;"
            },
            onclick: move |e| { onclick.call(e); },
            if active { "✓ {label}" } else { "{label}" }
        }
    }
}
