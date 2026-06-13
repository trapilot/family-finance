use dioxus::prelude::*;
use rust_decimal::Decimal;

use crate::app::AppState;
use crate::models::Transaction;
use crate::repository::TransactionRepo;
use crate::services::ReportService;
use crate::views::{current_month, fmt_currency, fmt_gold, fmt_usd, fmt_vnd, month_label, ts_to_date_str};

// ─── Dashboard ────────────────────────────────────────────────────────────────

#[component]
pub fn Dashboard() -> Element {
    let state = use_context::<AppState>();
    let (year, month) = current_month();

    // ── Data signals ──
    let net_worth = use_signal(|| crate::services::report_service::NetWorth {
        vnd: Decimal::ZERO,
        usd: Decimal::ZERO,
        gold: Decimal::ZERO,
    });
    let summary   = use_signal(|| crate::services::report_service::MonthlySummary {
        year, month,
        income: Decimal::ZERO, expense: Decimal::ZERO,
        savings: Decimal::ZERO, savings_rate: Decimal::ZERO,
        net_transfers: Decimal::ZERO,
    });
    let alerts    = use_signal(|| vec![]);
    let recent    = use_signal(|| vec![]);

    // ── Load on mount ──
    use_effect({
        let state = state.clone();
        let mut net_worth = net_worth.clone();
        let mut summary   = summary.clone();
        let mut alerts    = alerts.clone();
        let mut recent    = recent.clone();
        move || {
            state.with_conn(|conn| {
                if let Ok(nw) = ReportService::net_worth(conn) {
                    net_worth.set(nw);
                }
                if let Ok(s) = ReportService::monthly_summary(conn, year, month) {
                    summary.set(s);
                }
                if let Ok(a) = ReportService::overspending_alerts(conn, year, month) {
                    alerts.set(a);
                }
                if let Ok(txns) = TransactionRepo::list(conn, &crate::repository::transaction_repo::TxnFilter {
                    limit: Some(5),
                    ..Default::default()
                }) {
                    recent.set(txns);
                }
            });
        }
    });

    let s = summary.read();

    rsx! {
        div {
            style: "padding:16px; display:flex; flex-direction:column; gap:16px;",

            // ── Header ──
            div {
                style: "display:flex; justify-content:space-between; align-items:center;",
                h1 { style: "margin:0; font-size:20px; font-weight:700; color:#1f2937;", "Family Finance" }
                span { style: "font-size:13px; color:#6b7280;", "{month_label(year, month)}" }
            }

            // ── Net Worth Cards ──
            div {
                style: "display:grid; grid-template-columns:repeat(3,1fr); gap:10px;",
                NetWorthCard { label: "VND", value: fmt_vnd(net_worth.read().vnd),    color: "#6366f1" }
                NetWorthCard { label: "USD", value: fmt_usd(net_worth.read().usd),    color: "#10b981" }
                NetWorthCard { label: "GOLD", value: fmt_gold(net_worth.read().gold), color: "#f59e0b" }
            }

            // ── Monthly Summary ──
            div {
                style: "background:#fff; border-radius:12px; padding:16px; box-shadow:0 1px 3px rgba(0,0,0,.08);",
                h2 { style: "margin:0 0 12px; font-size:15px; font-weight:600; color:#374151;", "This Month" }
                div {
                    style: "display:grid; grid-template-columns:1fr 1fr; gap:10px;",
                    SummaryRow { label: "Income",       value: fmt_vnd(s.income),       color: "#10b981" }
                    SummaryRow { label: "Expense",      value: fmt_vnd(s.expense),      color: "#ef4444" }
                    SummaryRow { label: "Savings",      value: fmt_vnd(s.savings),      color: "#6366f1" }
                    SummaryRow { label: "Savings Rate", value: format!("{:.1}%", s.savings_rate), color: "#f59e0b" }
                }
            }

            // ── Overspending Alerts ──
            if !alerts.read().is_empty() {
                div {
                    style: "background:#fff; border-radius:12px; padding:16px; box-shadow:0 1px 3px rgba(0,0,0,.08);",
                    h2 { style: "margin:0 0 10px; font-size:15px; font-weight:600; color:#ef4444;", "⚠️ Overspending" }
                    for alert in alerts.read().iter() {
                        div {
                            style: "display:flex; justify-content:space-between; align-items:center; padding:6px 0; border-bottom:1px solid #f3f4f6;",
                            span { style: "font-size:13px; color:#374151;", "{alert.category_name}" }
                            div {
                                style: "text-align:right;",
                                div { style: "font-size:12px; color:#ef4444; font-weight:600;", "+{fmt_vnd(alert.overage)}" }
                                div { style: "font-size:11px; color:#9ca3af;", "{fmt_vnd(alert.spent)} / {fmt_vnd(alert.budget)}" }
                            }
                        }
                    }
                }
            }

            // ── Recent Transactions ──
            div {
                style: "background:#fff; border-radius:12px; padding:16px; box-shadow:0 1px 3px rgba(0,0,0,.08);",
                h2 { style: "margin:0 0 10px; font-size:15px; font-weight:600; color:#374151;", "Recent" }
                if recent.read().is_empty() {
                    p { style: "font-size:13px; color:#9ca3af; text-align:center; padding:16px 0;", "No transactions yet" }
                }
                for txn in recent.read().iter() {
                    RecentTxnRow { txn: txn.clone() }
                }
            }
        }
    }
}

// ─── Sub-components ───────────────────────────────────────────────────────────

#[component]
fn NetWorthCard(label: &'static str, value: String, color: &'static str) -> Element {
    rsx! {
        div {
            style: "background:#fff; border-radius:12px; padding:12px; box-shadow:0 1px 3px rgba(0,0,0,.08); border-top:3px solid {color};",
            div { style: "font-size:10px; font-weight:600; color:#9ca3af; text-transform:uppercase; margin-bottom:4px;", "{label}" }
            div { style: "font-size:13px; font-weight:700; color:#1f2937; word-break:break-all;", "{value}" }
        }
    }
}

#[component]
fn SummaryRow(label: &'static str, value: String, color: &'static str) -> Element {
    rsx! {
        div {
            style: "background:#f9fafb; border-radius:8px; padding:10px;",
            div { style: "font-size:11px; color:#6b7280; margin-bottom:2px;", "{label}" }
            div { style: "font-size:14px; font-weight:700; color:{color};", "{value}" }
        }
    }
}

#[component]
fn RecentTxnRow(txn: Transaction) -> Element {
    let (icon, color) = match txn.txn_type {
        crate::models::TransactionType::Income    => ("↑", "#10b981"),
        crate::models::TransactionType::Expense   => ("↓", "#ef4444"),
        crate::models::TransactionType::Transfer  => ("⇄", "#6366f1"),
        crate::models::TransactionType::InvestBuy  => ("📈", "#f59e0b"),
        crate::models::TransactionType::InvestSell => ("📉", "#8b5cf6"),
    };
    let amount_str = fmt_currency(txn.amount, &txn.currency);
    let date_str   = ts_to_date_str(txn.txn_date);

    rsx! {
        div {
            style: "display:flex; justify-content:space-between; align-items:center; padding:8px 0; border-bottom:1px solid #f3f4f6;",
            div {
                style: "display:flex; align-items:center; gap:10px;",
                span {
                    style: "width:32px; height:32px; border-radius:50%; background:#f3f4f6; display:flex; align-items:center; justify-content:center; font-size:16px;",
                    "{icon}"
                }
                div {
                    div { style: "font-size:13px; font-weight:500; color:#374151;", "{txn.txn_type}" }
                    div { style: "font-size:11px; color:#9ca3af;", "{date_str}" }
                }
            }
            span { style: "font-size:13px; font-weight:600; color:{color};", "{amount_str}" }
        }
    }
}
