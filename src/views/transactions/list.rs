use dioxus::prelude::*;

use crate::app::AppState;
use crate::models::{Transaction, TransactionType};
use crate::repository::transaction_repo::TxnFilter;
use crate::repository::TransactionRepo;
use crate::services::TransactionService;
use crate::views::{fmt_currency, ts_to_date_str};
use super::TxnView;

#[component]
pub fn TransactionList(view: Signal<TxnView>) -> Element {
    let state = use_context::<AppState>();

    let mut txns        = use_signal(|| vec![]);
    let mut filter_type = use_signal(|| None::<TransactionType>);
    let mut search_note = use_signal(|| String::new());
    let mut show_confirm_delete = use_signal(|| None::<String>); // holds txn id

    // ── Load ──
    // let mut load = {
    //     let state = state.clone();
    //     let mut txns = txns.clone();
    //     let filter_type = filter_type.clone();
    //     move || {
    //         let ft = filter_type.read().clone();
    //         state.with_conn(|conn| {
    //             if let Ok(list) = TransactionRepo::list(conn, &TxnFilter {
    //                 txn_type: ft,
    //                 limit: Some(100),
    //                 ..Default::default()
    //             }) {
    //                 txns.set(list);
    //             }
    //         });
    //     }
    // };

    let mut txns_effect = txns.clone();
    let state_effect = state.clone();

    use_effect(move || {
        let ft = filter_type.read().clone();

        state_effect.with_conn(|conn| {
            if let Ok(list) = TransactionRepo::list(conn, &TxnFilter {
                txn_type: ft,
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
                h1 { style: "margin:0; font-size:20px; font-weight:700; color:#1f2937;", "Transactions" }
                button {
                    style: "background:#6366f1; color:#fff; border:none; border-radius:20px; padding:8px 16px; font-size:14px; font-weight:600; cursor:pointer;",
                    onclick: move |_| { view.set(TxnView::Add); },
                    "+ Add"
                }
            }

            // ── Search ──
            input {
                style: "border:1px solid #e5e7eb; border-radius:8px; padding:10px 12px; font-size:14px; outline:none; background:#fff;",
                r#type: "text",
                placeholder: "Search by note…",
                value: "{search_note.read()}",
                oninput: move |e| {
                    search_note.set(e.value());
                },
            }

            // ── Type Filter ──
            div {
                style: "display:flex; gap:6px; overflow-x:auto; padding-bottom:4px;",
                FilterChip {
                    label: "All",
                    active: filter_type.read().is_none(),
                    onclick: move |_| {
                        filter_type.set(None);
                    }
                }
                for (tt, label) in [
                    (TransactionType::Income,    "Income"),
                    (TransactionType::Expense,   "Expense"),
                    (TransactionType::Transfer,  "Transfer"),
                    (TransactionType::InvestBuy, "Buy"),
                    (TransactionType::InvestSell,"Sell"),
                ] {
                    {
                        let tt_clone = tt.clone();
                        rsx! {
                            FilterChip {
                                label,
                                active: filter_type.read().as_ref() == Some(&tt),
                                onclick: move |_| {
                                    filter_type.set(Some(tt_clone.clone()));
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
                    "No transactions found"
                }
            }

            for txn in visible.iter() {
                TxnRow {
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
                        h3 { style: "margin:0 0 8px; font-size:17px;", "Delete transaction?" }
                        p  { style: "margin:0 0 20px; font-size:14px; color:#6b7280;", "This will reverse all balance changes." }
                        div {
                            style: "display:flex; gap:10px;",
                            button {
                                style: "flex:1; padding:10px; border:1px solid #e5e7eb; border-radius:8px; background:#fff; cursor:pointer; font-size:14px;",
                                onclick: move |_| { show_confirm_delete.set(None); },
                                "Cancel"
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
                                        
                                        let ft = filter_type.read().clone();

                                        state.with_conn(|conn| {
                                            if let Ok(list) = TransactionRepo::list(conn, &TxnFilter {
                                                txn_type: ft,
                                                limit: Some(100),
                                                ..Default::default()
                                            }) {
                                                txns.set(list);
                                            }
                                        });
                                        
                                    }
                                },
                                "Delete"
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
    on_edit: EventHandler<MouseEvent>,
    on_delete: EventHandler<MouseEvent>,
) -> Element {
    let (icon, amount_color) = match txn.txn_type {
        TransactionType::Income     => ("↑", "#10b981"),
        TransactionType::Expense    => ("↓", "#ef4444"),
        TransactionType::Transfer   => ("⇄", "#6366f1"),
        TransactionType::InvestBuy  => ("📈", "#f59e0b"),
        TransactionType::InvestSell => ("📉", "#8b5cf6"),
    };
    let amount_str = fmt_currency(txn.amount, &txn.currency);
    let date_str   = ts_to_date_str(txn.txn_date);
    let type_label = format!("{:?}", txn.txn_type).to_lowercase().replace("invest", "");
    let note       = txn.note.clone().unwrap_or_default();

    rsx! {
        div {
            style: "background:#fff; border-radius:12px; padding:12px; box-shadow:0 1px 3px rgba(0,0,0,.06); display:flex; align-items:center; gap:12px;",

            span {
                style: "width:36px; height:36px; border-radius:50%; background:#f3f4f6; display:flex; align-items:center; justify-content:center; font-size:18px; flex-shrink:0;",
                "{icon}"
            }

            div { style: "flex:1; min-width:0;",
                div { style: "font-size:13px; font-weight:600; color:#374151; text-transform:capitalize;", "{type_label}" }
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
            "{label}"
        }
    }
}
