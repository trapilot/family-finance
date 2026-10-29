use dioxus::prelude::*;
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::app::AppState;
use crate::i18n::use_i18n;
use crate::models::{Transaction, TransactionType};
use crate::repository::transaction_repo::{month_bounds, TxnFilter};
use crate::repository::{CategoryRepo, TransactionRepo, WalletRepo};
use crate::views::{fmt_currency, fmt_vnd, month_label, ts_to_date_str};
use super::expense::EmptyState;

/// Transactions behind a report number: one type over a range of whole months
#[derive(Clone, PartialEq, Debug)]
pub struct TypeDrill {
    pub txn_type: TransactionType,
    /// (year, month), inclusive
    pub from: (i32, u32),
    pub to:   (i32, u32),
}

impl TypeDrill {
    pub fn month(txn_type: TransactionType, year: i32, month: u32) -> Self {
        Self { txn_type, from: (year, month), to: (year, month) }
    }
}

/// Open the drill-down from any report (provided by `Reports`)
pub fn use_open_drill() -> Signal<Option<TypeDrill>> {
    use_context::<Signal<Option<TypeDrill>>>()
}

#[component]
pub fn TypeTransactions(drill: TypeDrill, on_back: EventHandler<()>) -> Element {
    let t = use_i18n();
    let state = use_context::<AppState>();

    // Same bounds as the report sums (`sum_by_type_in_month`), so the total matches
    let (txns, wallets, categories) = state.with_conn(|conn| {
        let filter = TxnFilter {
            txn_type:  Some(drill.txn_type.clone()),
            date_from: Some(month_bounds(drill.from.0, drill.from.1).0),
            date_to:   Some(month_bounds(drill.to.0, drill.to.1).1),
            ..Default::default()
        };
        let wallets: HashMap<String, String> = WalletRepo::list_active(conn).unwrap_or_default()
            .into_iter().map(|w| (w.id, w.name)).collect();
        let categories: HashMap<String, String> = CategoryRepo::list(conn).unwrap_or_default()
            .into_iter().map(|c| (c.id, format!("{} {}", c.icon.unwrap_or_default(), t.tr(&c.name)))).collect();
        (TransactionRepo::list(conn, &filter).unwrap_or_default(), wallets, categories)
    });
    let total: Decimal = txns.iter().map(|tx| tx.amount).sum();

    let type_label = t.code(&drill.txn_type.to_string());
    let period = if drill.from == drill.to {
        month_label(drill.from.0, drill.from.1)
    } else {
        format!("{} – {}", month_label(drill.from.0, drill.from.1), month_label(drill.to.0, drill.to.1))
    };
    let color = type_color(&drill.txn_type);

    rsx! {
        div {
            style: "display:flex; flex-direction:column; height:100%;",

            // ── Top bar ──
            div {
                style: "display:flex; align-items:center; gap:12px; padding:14px 16px; background:#fff; border-bottom:1px solid #f3f4f6;",
                button {
                    style: "border:none; background:transparent; font-size:22px; cursor:pointer; padding:0; flex-shrink:0;",
                    onclick: move |_| on_back.call(()),
                    "←"
                }
                div { style: "flex:1; min-width:0;",
                    div { style: "font-size:17px; font-weight:700; color:#1f2937; text-transform:capitalize;", "{type_label}" }
                    div { style: "font-size:12px; color:#6b7280;", "{period}" }
                }
            }

            div {
                style: "flex:1; overflow-y:auto; padding:16px; display:flex; flex-direction:column; gap:10px;",

                // ── Total ──
                div {
                    style: "background:#fff; border-radius:12px; padding:14px; box-shadow:0 1px 3px rgba(0,0,0,.06); display:flex; justify-content:space-between; align-items:baseline;",
                    span { style: "font-size:12px; color:#6b7280;",
                        {t.t("{n} transactions").replace("{n}", &txns.len().to_string())}
                    }
                    span { style: "font-size:20px; font-weight:800; color:{color};", "{fmt_vnd(total)}" }
                }

                if txns.is_empty() {
                    EmptyState { msg: t.t("No transactions in this period") }
                }
                for txn in txns {
                    TxnLine {
                        key: "{txn.id}",
                        detail: detail_line(&txn, &wallets, &categories),
                        txn,
                    }
                }
            }
        }
    }
}

/// Read-only row: what it was (category / wallets), note, date, amount
#[component]
fn TxnLine(txn: Transaction, detail: String) -> Element {
    let note = txn.note.clone().unwrap_or_default();
    let color = type_color(&txn.txn_type);
    rsx! {
        div {
            style: "background:#fff; border-radius:12px; padding:12px; box-shadow:0 1px 3px rgba(0,0,0,.06); display:flex; align-items:center; gap:12px;",
            div { style: "flex:1; min-width:0;",
                div { style: "font-size:13px; font-weight:600; color:#374151; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;", "{detail}" }
                if !note.is_empty() {
                    div { style: "font-size:12px; color:#6b7280; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;", "{note}" }
                }
                div { style: "font-size:11px; color:#9ca3af;", "{ts_to_date_str(txn.txn_date)}" }
            }
            div { style: "font-size:14px; font-weight:700; color:{color}; flex-shrink:0;",
                "{fmt_currency(txn.amount, &txn.currency)}"
            }
        }
    }
}

fn detail_line(txn: &Transaction, wallets: &HashMap<String, String>, categories: &HashMap<String, String>) -> String {
    let wallet = |id: &str| wallets.get(id).cloned().unwrap_or_default();
    match txn.txn_type {
        TransactionType::Expense => {
            let category = txn.category_id.as_deref().and_then(|c| categories.get(c)).cloned().unwrap_or_default();
            format!("{category} · {}", wallet(&txn.wallet_id))
        }
        TransactionType::Transfer => {
            format!("{} → {}", wallet(&txn.wallet_id), txn.to_wallet_id.as_deref().map(wallet).unwrap_or_default())
        }
        _ => wallet(&txn.wallet_id),
    }
}

fn type_color(txn_type: &TransactionType) -> &'static str {
    match txn_type {
        TransactionType::Income  => "#10b981",
        TransactionType::Expense => "#ef4444",
        _                        => "#6366f1",
    }
}
