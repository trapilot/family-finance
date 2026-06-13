use dioxus::prelude::*;
use rust_decimal::Decimal;

use crate::app::AppState;
use crate::services::{report_service::ExpenseByCategoryItem, ReportService};
use crate::views::{current_month, fmt_vnd, month_label};

#[component]
pub fn ExpenseReport() -> Element {
    let state = use_context::<AppState>();
    let (y0, m0) = current_month();

    let mut year  = use_signal(|| y0);
    let mut month = use_signal(|| m0);
    let items = use_signal(|| vec![]);
    let total = use_signal(|| Decimal::ZERO);

    let load = {
        let state = state.clone();
        let year  = year.clone();
        let month = month.clone();
        let mut items = items.clone();
        let mut total = total.clone();
        move || {
            state.with_conn(|conn| {
                let (y, m) = (*year.read(), *month.read());
                if let Ok(list) = ReportService::expense_by_category(conn, y, m) {
                    let t: Decimal = list.iter().map(|i| i.amount).sum();
                    total.set(t);
                    items.set(list);
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

    rsx! {
        div {
            style: "display:flex; flex-direction:column; gap:14px;",

            // ── Month navigator ──
            MonthPicker {
                year: *year.read(),
                month: *month.read(),
                on_prev: {
                    let mut load = load.clone();
                    move |_| {
                        let (y, m) = (*year.read(), *month.read());
                        if m == 1 { year.set(y - 1); month.set(12); }
                        else       { month.set(m - 1); }
                        load();
                    }
                },
                on_next: {
                    let mut load = load.clone();
                    move |_| {
                        let (y, m) = (*year.read(), *month.read());
                        if m == 12 { year.set(y + 1); month.set(1); }
                        else        { month.set(m + 1); }
                        load();
                    }
                }
            }

            // ── Total ──
            div {
                style: "background:#fff; border-radius:12px; padding:14px; box-shadow:0 1px 3px rgba(0,0,0,.06);",
                div { style: "font-size:11px; color:#9ca3af; text-transform:uppercase; margin-bottom:4px;", "Total Expense" }
                div { style: "font-size:24px; font-weight:800; color:#ef4444;", "{fmt_vnd(*total.read())}" }
            }

            // ── Category breakdown ──
            if items.read().is_empty() {
                EmptyState { msg: "No expenses this month" }
            }

            for item in items.read().iter() {
                CategoryBar { item: item.clone(), total: *total.read() }
            }
        }
    }
}

// ─── CategoryBar ─────────────────────────────────────────────────────────────

#[component]
fn CategoryBar(item: ExpenseByCategoryItem, total: Decimal) -> Element {
    use rust_decimal::prelude::ToPrimitive;

    let pct = item.share_pct.to_f64().unwrap_or(0.0).max(0.0).min(100.0);
    let bar_color = if item.is_over_budget { "#ef4444" } else { "#6366f1" };
    let icon  = item.icon.as_deref().unwrap_or("📦");
    let color = item.color.as_deref().unwrap_or("#6366f1");

    rsx! {
        div {
            style: "background:#fff; border-radius:12px; padding:14px; box-shadow:0 1px 3px rgba(0,0,0,.06);",

            div {
                style: "display:flex; align-items:center; gap:10px; margin-bottom:8px;",
                span {
                    style: "width:36px; height:36px; border-radius:8px; background:{color}22; display:flex; align-items:center; justify-content:center; font-size:18px; flex-shrink:0;",
                    "{icon}"
                }
                div { style: "flex:1;",
                    div { style: "display:flex; justify-content:space-between; align-items:baseline;",
                        span { style: "font-size:14px; font-weight:600; color:#374151;", "{item.category_name}" }
                        span { style: "font-size:14px; font-weight:700; color:#1f2937;", "{fmt_vnd(item.amount)}" }
                    }
                    div { style: "display:flex; justify-content:space-between; margin-top:1px;",
                        span { style: "font-size:11px; color:#9ca3af;", "{pct:.1}% of total" }
                        if item.is_over_budget {
                            span {
                                style: "font-size:11px; color:#ef4444; font-weight:600;",
                                "⚠️ Over budget"
                            }
                        } else if let Some(budget) = item.budget_amount {
                            span {
                                style: "font-size:11px; color:#9ca3af;",
                                "Budget: {fmt_vnd(budget)}"
                            }
                        }
                    }
                }
            }

            // Progress bar
            div {
                style: "height:6px; background:#f3f4f6; border-radius:3px; overflow:hidden;",
                div {
                    style: "height:100%; width:{pct}%; background:{bar_color}; border-radius:3px; transition:width .3s;",
                }
            }

            // Budget bar (if set)
            if let Some(budget) = item.budget_amount {
                if budget > Decimal::ZERO {
                    {
                        use rust_decimal::prelude::ToPrimitive;
                        let budget_pct = (item.amount / budget * Decimal::from(100))
                            .to_f64().unwrap_or(0.0).min(100.0);
                        
                        let budget_style = format!(
                            "height:100%; width:{}%; background:{}; border-radius:2px;",
                            budget_pct,
                            color
                        );

                        rsx! {
                            div { style: "margin-top:4px;",
                                div { style: "font-size:10px; color:#9ca3af; margin-bottom:2px;",
                                    "Budget usage: {budget_pct:.0}%"
                                }
                                div {
                                    style: "height:4px; background:#f3f4f6; border-radius:2px; overflow:hidden;",
                                    div {
                                        style: "{budget_style}",
                                        background: if item.is_over_budget { "#ef4444" } else { "#10b981" },
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ─── Shared helpers ───────────────────────────────────────────────────────────

#[component]
pub fn MonthPicker(
    year: i32,
    month: u32,
    on_prev: EventHandler<MouseEvent>,
    on_next: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div {
            style: "display:flex; align-items:center; justify-content:space-between; background:#fff; border-radius:12px; padding:10px 14px; box-shadow:0 1px 3px rgba(0,0,0,.06);",
            button {
                style: "border:none; background:#f3f4f6; border-radius:8px; padding:6px 12px; cursor:pointer; font-size:16px; color:#374151;",
                onclick: move |e| { on_prev.call(e); },
                "‹"
            }
            span { style: "font-size:15px; font-weight:700; color:#1f2937;", "{month_label(year, month)}" }
            button {
                style: "border:none; background:#f3f4f6; border-radius:8px; padding:6px 12px; cursor:pointer; font-size:16px; color:#374151;",
                onclick: move |e| { on_next.call(e); },
                "›"
            }
        }
    }
}

#[component]
pub fn EmptyState(msg: &'static str) -> Element {
    rsx! {
        div {
            style: "text-align:center; padding:40px 0; color:#9ca3af;",
            p { style: "font-size:32px; margin:0;", "📭" }
            p { style: "font-size:14px; margin:8px 0 0;", "{msg}" }
        }
    }
}
