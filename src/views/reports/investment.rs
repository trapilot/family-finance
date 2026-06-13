use dioxus::prelude::*;
use rust_decimal::Decimal;

use crate::app::AppState;
use crate::services::{report_service::InvestmentPnlItem, ReportService};
use crate::views::fmt_currency;
use crate::models::Currency;
use super::expense::EmptyState;

#[component]
pub fn InvestmentReport() -> Element {
    let state = use_context::<AppState>();
    let mut items = use_signal(|| vec![]);

    // ── Load data (runs once on mount) ──
    use_effect(move || {
        state.with_conn(|conn| {
            if let Ok(list) = ReportService::investment_pnl(conn) {
                items.set(list);
            }
        });
    });

    let data = items.read();

    // ── Aggregations ──
    let total_cost: Decimal = data.iter().map(|i| i.cost_basis).sum();

    let total_mv: Decimal = data
        .iter()
        .filter_map(|i| i.market_value)
        .fold(Decimal::ZERO, |acc, v| acc + v);

    let total_unrealized: Decimal = data
        .iter()
        .filter_map(|i| i.unrealized_pnl)
        .fold(Decimal::ZERO, |acc, v| acc + v);

    let total_realized: Decimal = data
        .iter()
        .map(|i| i.realized_pnl)
        .sum();

    // ── Precomputed formatted values ──
    let unrealized_value = format!(
        "{}{}",
        if total_unrealized >= Decimal::ZERO { "+" } else { "" },
        fmt_currency(total_unrealized, &Currency::VND)
    );

    let realized_value = format!(
        "{}{}",
        if total_realized >= Decimal::ZERO { "+" } else { "" },
        fmt_currency(total_realized, &Currency::VND)
    );

    let unrealized_color = if total_unrealized >= Decimal::ZERO {
        "#6ee7b7"
    } else {
        "#fca5a5"
    };

    let realized_color = if total_realized >= Decimal::ZERO {
        "#fde68a"
    } else {
        "#fca5a5"
    };

    rsx! {
        div {
            style: "display:flex; flex-direction:column; gap:14px;",

            // ── Overview ──
            div {
                style: "background:linear-gradient(135deg, #0f172a, #1e293b); border-radius:12px; padding:16px; color:#fff;",

                div {
                    style: "font-size:11px; opacity:.6; text-transform:uppercase; letter-spacing:.5px; margin-bottom:12px;",
                    "Investment Overview"
                }

                div {
                    style: "display:grid; grid-template-columns:1fr 1fr; gap:10px;",

                    InvStat {
                        label: "Cost Basis",
                        value: fmt_currency(total_cost, &Currency::VND),
                        color: "#a5b4fc"
                    }

                    InvStat {
                        label: "Market Value",
                        value: fmt_currency(total_mv, &Currency::VND),
                        color: "#6ee7b7"
                    }

                    InvStat {
                        label: "Unrealized P&L",
                        value: unrealized_value,
                        color: unrealized_color
                    }

                    InvStat {
                        label: "Realized P&L",
                        value: realized_value,
                        color: realized_color
                    }
                }
            }

            // ── Body ──
            {
                if data.is_empty() {
                    rsx! {
                        EmptyState {
                            msg: "No investment holdings yet"
                        }
                    }
                } else {
                    rsx! {
                        for item in data.iter() {
                            InvestmentCard {
                                item: item.clone()
                            }
                        }
                    }
                }
            }
        }
    }
}

// ─── InvestmentCard ───────────────────────────────────────────────────────────

#[component]
fn InvestmentCard(item: InvestmentPnlItem) -> Element {
    let unr = item.unrealized_pnl.unwrap_or(Decimal::ZERO);
    let unr_color  = if unr >= Decimal::ZERO { "#10b981" } else { "#ef4444" };
    let real_color = if item.realized_pnl >= Decimal::ZERO { "#10b981" } else { "#ef4444" };

    let unr_sign  = if unr >= Decimal::ZERO { "+" } else { "" };
    let real_sign = if item.realized_pnl >= Decimal::ZERO { "+" } else { "" };

    let pct_str = item.unrealized_pnl_pct
        .map(|p| {
            let sign = if p >= Decimal::ZERO { "+" } else { "" };
            format!("{sign}{p:.2}%")
        })
        .unwrap_or("—".into());

    rsx! {
        div {
            style: "background:#fff; border-radius:12px; padding:14px; box-shadow:0 1px 3px rgba(0,0,0,.06);",

            // Symbol + name
            div {
                style: "display:flex; align-items:center; justify-content:space-between; margin-bottom:10px;",
                div {
                    div { style: "font-size:16px; font-weight:800; color:#1f2937;", "{item.symbol}" }
                    if let Some(ref name) = item.name {
                        div { style: "font-size:12px; color:#9ca3af;", "{name}" }
                    }
                }
                div { style: "text-align:right;",
                    div { style: "font-size:16px; font-weight:700; color:{unr_color};",
                        "{unr_sign}{fmt_currency(unr, &Currency::VND)}"
                    }
                    div { style: "font-size:12px; color:{unr_color};", "{pct_str}" }
                }
            }

            // Stats grid
            div {
                style: "display:grid; grid-template-columns:repeat(3,1fr); gap:6px;",
                PnlCell { label: "Quantity",    value: format!("{:.4}", item.quantity),           color: "#374151" }
                PnlCell { label: "Avg Cost",    value: format!("{:.0}", item.avg_buy_price),       color: "#374151" }
                PnlCell { label: "Last Price",  value: item.last_price.map(|p| format!("{p:.0}")).unwrap_or("—".into()), color: "#374151" }
                PnlCell { label: "Cost Basis",  value: fmt_currency(item.cost_basis, &Currency::VND),  color: "#6b7280" }
                PnlCell { label: "Mkt Value",   value: item.market_value.map(|v| fmt_currency(v, &Currency::VND)).unwrap_or("—".into()), color: "#6b7280" }
                PnlCell { label: "Realized P&L", value: format!("{real_sign}{}", fmt_currency(item.realized_pnl, &Currency::VND)), color: real_color }
            }
        }
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

#[component]
fn InvStat(label: &'static str, value: String, color: &'static str) -> Element {
    rsx! {
        div {
            div {
                style: "font-size:10px; opacity:.6; margin-bottom:2px;",
                "{label}"
            }

            div {
                style: "font-size:13px; font-weight:700; color:{color};",
                "{value}"
            }
        }
    }
}

#[component]
fn PnlCell(label: &'static str, value: String, color: &'static str) -> Element {
    rsx! {
        div {
            style: "background:#f9fafb; border-radius:6px; padding:6px 8px;",
            div { style: "font-size:10px; color:#9ca3af; margin-bottom:1px;", "{label}" }
            div { style: "font-size:12px; font-weight:600; color:{color};", "{value}" }
        }
    }
}
