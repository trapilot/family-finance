use dioxus::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::app::AppState;
use crate::services::{report_service::MonthlySummary, ReportService};
use crate::views::{fmt_vnd, month_label};
use super::expense::EmptyState;

#[component]
pub fn TimelineReport() -> Element {
    let state    = use_context::<AppState>();
    let mut months   = use_signal(|| 6u32);
    let summaries = use_signal(|| vec![]);
    let mut selected  = use_signal(|| None::<usize>);

    let load = {
        let state = state.clone();
        let months = months.clone();
        let mut summaries = summaries.clone();
        move || {
            state.with_conn(|conn| {
                if let Ok(list) = ReportService::income_expense_timeline(conn, *months.read()) {
                    summaries.set(list);
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

    let data = summaries.read();
    let max_val = data.iter()
        .flat_map(|s| [s.income, s.expense])
        .max()
        .unwrap_or(Decimal::ONE)
        .max(Decimal::ONE);
    let max_f = max_val.to_f64().unwrap_or(1.0);

    // Bar chart dimensions
    let chart_h:  f64 = 160.0;
    let bar_w:    f64 = 24.0;
    let gap:      f64 = 8.0;
    let group_w:  f64 = bar_w * 2.0 + gap + 16.0;
    let n         = data.len();
    let chart_w   = group_w * n as f64 + 20.0;

    rsx! {
        div {
            style: "display:flex; flex-direction:column; gap:14px;",

            // ── Period selector ──
            div {
                style: "display:flex; gap:8px;",
                for (n, label) in [(3u32, "3M"), (6, "6M"), (12, "12M")] {
                    {
                        let is_active = *months.read() == n;
                        let mut load = load.clone();
                        rsx! {
                            button {
                                style: if is_active {
                                    "flex:1; padding:8px; border:none; border-radius:8px; background:#6366f1; color:#fff; font-size:13px; font-weight:600; cursor:pointer;"
                                } else {
                                    "flex:1; padding:8px; border:1px solid #e5e7eb; border-radius:8px; background:#fff; color:#6b7280; font-size:13px; cursor:pointer;"
                                },
                                onclick: move |_| {
                                    months.set(n);
                                    load();
                                },
                                "{label}"
                            }
                        }
                    }
                }
            }

            if data.is_empty() {
                EmptyState { msg: "Not enough data yet" }
            } else {
                // ── SVG Bar Chart ──
                div {
                    style: "background:#fff; border-radius:12px; padding:14px; box-shadow:0 1px 3px rgba(0,0,0,.06); overflow-x:auto;",
                    svg {
                        width: "{chart_w}",
                        height: "{chart_h + 40.0}",
                        view_box: "0 0 {chart_w} {chart_h + 40.0}",

                        // Y-axis gridlines
                        for i in 0..=4 {
                            {
                                let y = chart_h - (i as f64 / 4.0) * chart_h;
                                rsx! {
                                    line {
                                        x1: "0", y1: "{y}",
                                        x2: "{chart_w}", y2: "{y}",
                                        stroke: "#f3f4f6", stroke_width: "1"
                                    }
                                }
                            }
                        }

                        // Bars
                        for (i, s) in data.iter().enumerate() {
                            {
                                let x_base = i as f64 * group_w + 10.0;
                                let income_h  = (s.income.to_f64().unwrap_or(0.0) / max_f * chart_h).max(2.0);
                                let expense_h = (s.expense.to_f64().unwrap_or(0.0) / max_f * chart_h).max(2.0);
                                let ix = x_base;
                                let ex = x_base + bar_w + gap;
                                let is_sel = *selected.read() == Some(i);
                                rsx! {
                                    // Income bar
                                    rect {
                                        x: "{ix}", y: "{chart_h - income_h}",
                                        width: "{bar_w}", height: "{income_h}",
                                        rx: "3", fill: if is_sel { "#059669" } else { "#10b981" },
                                        onclick: move |_| {
                                            let cur = *selected.read();
                                            selected.set(if cur == Some(i) { None } else { Some(i) });
                                        },
                                        style: "cursor:pointer;"
                                    }
                                    // Expense bar
                                    rect {
                                        x: "{ex}", y: "{chart_h - expense_h}",
                                        width: "{bar_w}", height: "{expense_h}",
                                        rx: "3", fill: if is_sel { "#dc2626" } else { "#ef4444" },
                                        onclick: move |_| {
                                            let cur = *selected.read();
                                            selected.set(if cur == Some(i) { None } else { Some(i) });
                                        },
                                        style: "cursor:pointer;"
                                    }
                                    // Month label
                                    text {
                                        x: "{ix + bar_w}", y: "{chart_h + 14.0}",
                                        text_anchor: "middle",
                                        font_size: "9",
                                        fill: "#9ca3af",
                                        "{month_label(s.year, s.month)}"
                                    }
                                }
                            }
                        }
                    }

                    // Legend
                    div {
                        style: "display:flex; gap:16px; margin-top:8px; justify-content:center;",
                        div { style: "display:flex; align-items:center; gap:4px; font-size:12px; color:#6b7280;",
                            div { style: "width:12px; height:12px; background:#10b981; border-radius:2px;" }
                            "Income"
                        }
                        div { style: "display:flex; align-items:center; gap:4px; font-size:12px; color:#6b7280;",
                            div { style: "width:12px; height:12px; background:#ef4444; border-radius:2px;" }
                            "Expense"
                        }
                    }
                }

                // ── Selected month detail ──
                if let Some(idx) = *selected.read() {
                    if let Some(s) = data.get(idx) {
                        MonthDetailCard { summary: s.clone() }
                    }
                }

                // ── Table ──
                div {
                    style: "background:#fff; border-radius:12px; padding:14px; box-shadow:0 1px 3px rgba(0,0,0,.06);",
                    // Header row
                    div {
                        style: "display:grid; grid-template-columns:80px 1fr 1fr 1fr; gap:4px; padding:6px 0; border-bottom:1px solid #f3f4f6;",
                        span { style: "font-size:11px; font-weight:700; color:#9ca3af;", "Month" }
                        span { style: "font-size:11px; font-weight:700; color:#10b981; text-align:right;", "Income" }
                        span { style: "font-size:11px; font-weight:700; color:#ef4444; text-align:right;", "Expense" }
                        span { style: "font-size:11px; font-weight:700; color:#6366f1; text-align:right;", "Savings" }
                    }
                    for s in data.iter().rev() {
                        div {
                            style: "display:grid; grid-template-columns:80px 1fr 1fr 1fr; gap:4px; padding:8px 0; border-bottom:1px solid #f9fafb;",
                            span { style: "font-size:12px; color:#374151;", "{month_label(s.year, s.month)}" }
                            span { style: "font-size:12px; font-weight:600; color:#10b981; text-align:right;", "{fmt_vnd(s.income)}" }
                            span { style: "font-size:12px; font-weight:600; color:#ef4444; text-align:right;", "{fmt_vnd(s.expense)}" }
                            span {
                                style: if s.savings >= Decimal::ZERO {
                                    "font-size:12px; font-weight:600; color:#6366f1; text-align:right;"
                                } else {
                                    "font-size:12px; font-weight:600; color:#ef4444; text-align:right;"
                                },
                                "{fmt_vnd(s.savings)}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn MonthDetailCard(summary: MonthlySummary) -> Element {
    let savings_rate_color = if summary.savings_rate >= Decimal::ZERO {
        "#f59e0b"
    } else {
        "#ef4444"
    };

    rsx! {
        div {
            style: "background:#fff; border-radius:12px; padding:14px; box-shadow:0 1px 3px rgba(0,0,0,.06);",

            div {
                style: "font-size:13px; font-weight:700; color:#374151; margin-bottom:10px;",
                "{month_label(summary.year, summary.month)} — Detail"
            }

            div {
                style: "display:grid; grid-template-columns:1fr 1fr; gap:8px;",

                StatBox {
                    label: "Income",
                    value: fmt_vnd(summary.income),
                    color: "#10b981"
                }

                StatBox {
                    label: "Expense",
                    value: fmt_vnd(summary.expense),
                    color: "#ef4444"
                }

                StatBox {
                    label: "Savings",
                    value: fmt_vnd(summary.savings),
                    color: "#6366f1"
                }

                StatBox {
                    label: "Savings Rate",
                    value: format!("{:.1}%", summary.savings_rate),
                    color: savings_rate_color,
                }
            }
        }
    }
}

#[component]
fn StatBox(label: &'static str, value: String, color: &'static str) -> Element {
    rsx! {
        div {
            style: "background:#f9fafb; border-radius:8px; padding:10px;",
            div { style: "font-size:11px; color:#9ca3af; margin-bottom:2px;", "{label}" }
            div { style: "font-size:14px; font-weight:700; color:{color};", "{value}" }
        }
    }
}
