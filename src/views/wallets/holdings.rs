use dioxus::prelude::*;
use rust_decimal::Decimal;

use crate::app::AppState;
use crate::models::holding::AssetType;
use crate::services::{holding_service::HoldingWithPnl, HoldingService};
use crate::views::fmt_currency;
use crate::models::Currency;

#[component]
pub fn HoldingsView(wallet_id: String) -> Element {
    let state    = use_context::<AppState>();
    let holdings = use_signal(|| vec![]);
    let mut edit_price_id  = use_signal(|| None::<String>);
    let mut price_input    = use_signal(|| String::new());

    let load = {
        let state     = state.clone();
        let wallet_id = wallet_id.clone();
        let mut holdings = holdings.clone();
        move || {
            state.with_conn(|conn| {
                if let Ok(list) = HoldingService::portfolio_summary(conn, &wallet_id) {
                    holdings.set(list);
                }
            });
        }
    };

    use_effect({
        let mut load = load.clone();
        move || {
            load();
        }
    });

    let total_cost: Decimal = holdings.read().iter().map(|h| h.cost_basis).sum();
    let total_mv: Option<Decimal> = {
        let mvs: Vec<Decimal> = holdings.read().iter().filter_map(|h| h.market_value).collect();
        if mvs.len() == holdings.read().len() && !mvs.is_empty() {
            Some(mvs.iter().sum())
        } else {
            None
        }
    };
    let total_pnl: Option<Decimal> = total_mv.map(|mv| mv - total_cost);

    rsx! {
        div {
            style: "display:flex; flex-direction:column; gap:12px;",

            // ── Portfolio summary ──
            div {
                style: "background:linear-gradient(135deg, #1e1b4b, #312e81); border-radius:12px; padding:16px; color:#fff;",
                div { style: "font-size:11px; opacity:.7; text-transform:uppercase; letter-spacing:.5px; margin-bottom:8px;", "Portfolio" }
                div {
                    style: "display:grid; grid-template-columns:1fr 1fr; gap:10px;",
                    PortfolioStat {
                        label: "Cost Basis",
                        value: fmt_currency(total_cost, &Currency::VND),
                        color: "#a5b4fc"
                    }
                    PortfolioStat {
                        label: "Market Value",
                        value: total_mv.map(|v| fmt_currency(v, &Currency::VND)).unwrap_or("—".into()),
                        color: "#6ee7b7"
                    }
                    PortfolioStat {
                        label: "Unrealized P&L",
                        value: total_pnl.map(|p| {
                            let sign = if p >= Decimal::ZERO { "+" } else { "" };
                            format!("{sign}{}", fmt_currency(p, &Currency::VND))
                        }).unwrap_or("—".into()),
                        color: total_pnl.map(|p| if p >= Decimal::ZERO { "#6ee7b7" } else { "#fca5a5" }).unwrap_or("#a5b4fc")
                    }
                    PortfolioStat {
                        label: "Holdings",
                        value: holdings.read().len().to_string(),
                        color: "#fde68a"
                    }
                }
            }

            // ── Holdings list ──
            if holdings.read().is_empty() {
                div {
                    style: "text-align:center; padding:32px 0; color:#9ca3af; font-size:14px;",
                    p { style: "font-size:28px; margin:0;", "📈" }
                    p { style: "margin:8px 0 0;", "No holdings yet. Add a Buy transaction." }
                }
            }

            for item in holdings.read().iter() {
                HoldingCard {
                    item: item.clone(),
                    is_editing_price: *edit_price_id.read() == Some(item.holding.id.clone()),
                    price_input: price_input.read().clone(),
                    on_edit_price: {
                        let id    = item.holding.id.clone();
                        let price = item.holding.last_price.map(|p| format!("{p}")).unwrap_or_default();
                        move |_| {
                            edit_price_id.set(Some(id.clone()));
                            price_input.set(price.clone());
                        }
                    },
                    on_price_input: move |v| { price_input.set(v); },
                    on_price_save: {
                        let state = state.clone();
                        let id    = item.holding.id.clone();
                        let mut load  = load.clone();
                        move |_: MouseEvent| {
                            let price_str = price_input.read().clone();
                            if let Ok(price) = price_str.parse::<Decimal>() {
                                state.with_conn(|conn| {
                                    let _ = HoldingService::update_last_price(conn, &id, price);
                                });
                                edit_price_id.set(None);
                                load();
                            }
                        }
                    },
                    on_price_cancel: move |_| { edit_price_id.set(None); }
                }
            }
        }
    }
}

// ─── HoldingCard ─────────────────────────────────────────────────────────────

#[component]
fn HoldingCard(
    item: HoldingWithPnl,
    is_editing_price: bool,
    price_input: String,
    on_edit_price:   EventHandler<MouseEvent>,
    on_price_input:  EventHandler<String>,
    on_price_save:   EventHandler<MouseEvent>,
    on_price_cancel: EventHandler<MouseEvent>,
) -> Element {
    let h = &item.holding;
    let asset_color = match h.asset_type {
        AssetType::Stock  => "#6366f1",
        AssetType::Crypto => "#f59e0b",
    };
    let asset_icon = match h.asset_type {
        AssetType::Stock  => "📊",
        AssetType::Crypto => "🪙",
    };

    let pnl_color = item.unrealized_pnl
        .map(|p| if p >= Decimal::ZERO { "#10b981" } else { "#ef4444" })
        .unwrap_or("#6b7280");

    let pnl_str = item.unrealized_pnl
        .map(|p| {
            let sign = if p >= Decimal::ZERO { "+" } else { "" };
            format!("{sign}{p:.0}")
        })
        .unwrap_or("—".into());

    let pnl_pct_str = item.unrealized_pnl_pct()
        .map(|p| format!("{p:.2}%"))
        .unwrap_or("—".into());

    rsx! {
        div {
            style: "background:#fff; border-radius:12px; padding:14px; box-shadow:0 1px 3px rgba(0,0,0,.06);",

            // ── Symbol row ──
            div {
                style: "display:flex; align-items:center; gap:10px; margin-bottom:10px;",
                div {
                    style: "width:40px; height:40px; border-radius:10px; background:{asset_color}22; display:flex; align-items:center; justify-content:center; font-size:20px; flex-shrink:0;",
                    "{asset_icon}"
                }
                div { style: "flex:1;",
                    div { style: "font-size:16px; font-weight:700; color:#1f2937;", "{h.symbol}" }
                    if let Some(ref name) = h.name {
                        div { style: "font-size:12px; color:#9ca3af;", "{name}" }
                    }
                }
                div { style: "text-align:right;",
                    div { style: "font-size:14px; font-weight:700; color:{pnl_color};", "{pnl_str}" }
                    div { style: "font-size:12px; color:{pnl_color};", "{pnl_pct_str}" }
                }
            }

            // ── Stats grid ──
            div {
                style: "display:grid; grid-template-columns:repeat(3,1fr); gap:8px; margin-bottom:10px;",
                StatCell { label: "Qty",       value: format!("{:.4}", h.quantity) }
                StatCell { label: "Avg Cost",  value: format!("{:.0}", h.avg_buy_price) }
                StatCell { label: "Last Price", value: h.last_price.map(|p| format!("{p:.0}")).unwrap_or("—".into()) }
                StatCell { label: "Cost Basis", value: format!("{:.0}", item.cost_basis) }
                StatCell { label: "Mkt Value",  value: item.market_value.map(|v| format!("{v:.0}")).unwrap_or("—".into()) }
                StatCell { label: "Type",       value: format!("{}", h.asset_type) }
            }

            // ── Price update ──
            if is_editing_price {
                div {
                    style: "display:flex; gap:8px; align-items:center;",
                    input {
                        style: "flex:1; border:1px solid #6366f1; border-radius:8px; padding:8px 10px; font-size:14px; outline:none;",
                        r#type: "number",
                        placeholder: "New price",
                        value: "{price_input}",
                        oninput: move |e| { on_price_input.call(e.value()); },
                    }
                    button {
                        style: "padding:8px 14px; background:#6366f1; color:#fff; border:none; border-radius:8px; font-size:13px; font-weight:600; cursor:pointer;",
                        onclick: move |e| { on_price_save.call(e); },
                        "Save"
                    }
                    button {
                        style: "padding:8px 12px; background:#f3f4f6; color:#6b7280; border:none; border-radius:8px; font-size:13px; cursor:pointer;",
                        onclick: move |e| { on_price_cancel.call(e); },
                        "✕"
                    }
                }
            } else {
                button {
                    style: "width:100%; padding:8px; border:1px dashed #d1d5db; border-radius:8px; background:transparent; color:#6b7280; font-size:13px; cursor:pointer;",
                    onclick: move |e| { on_edit_price.call(e); },
                    "Update last price"
                }
            }
        }
    }
}

#[component]
fn PortfolioStat(label: &'static str, value: String, color: &'static str) -> Element {
    rsx! {
        div {
            div { style: "font-size:10px; opacity:.7; margin-bottom:2px;", "{label}" }
            div { style: "font-size:13px; font-weight:700; color:{color};", "{value}" }
        }
    }
}

#[component]
fn StatCell(label: &'static str, value: String) -> Element {
    rsx! {
        div {
            style: "background:#f9fafb; border-radius:6px; padding:6px 8px;",
            div { style: "font-size:10px; color:#9ca3af; margin-bottom:1px;", "{label}" }
            div { style: "font-size:12px; font-weight:600; color:#374151;", "{value}" }
        }
    }
}
