pub mod expense;
pub mod investment;
pub mod timeline;

use dioxus::prelude::*;
use expense::ExpenseReport;
use timeline::TimelineReport;
use investment::InvestmentReport;

#[derive(Clone, PartialEq)]
enum ReportTab { Expense, Timeline, Investment }

#[component]
pub fn Reports() -> Element {
    let mut tab = use_signal(|| ReportTab::Expense);

    rsx! {
        div {
            style: "display:flex; flex-direction:column; height:100%;",

            // ── Header ──
            div {
                style: "padding:16px 16px 0; background:#fff;",
                h1 { style: "margin:0 0 12px; font-size:20px; font-weight:700; color:#1f2937;", "Reports" }
                div {
                    style: "display:flex; gap:0; border-bottom:1px solid #f3f4f6;",
                    for (t, label) in [
                        (ReportTab::Expense,    "Expense"),
                        (ReportTab::Timeline,   "Timeline"),
                        (ReportTab::Investment, "Invest"),
                    ] {
                        {
                            let is_active = *tab.read() == t;
                            let t_clone = t.clone();
                            rsx! {
                                button {
                                    style: if is_active {
                                        "flex:1; padding:10px 0; border:none; border-bottom:2px solid #6366f1; background:#fff; font-size:13px; font-weight:600; color:#6366f1; cursor:pointer;"
                                    } else {
                                        "flex:1; padding:10px 0; border:none; border-bottom:2px solid transparent; background:#fff; font-size:13px; color:#9ca3af; cursor:pointer;"
                                    },
                                    onclick: move |_| { tab.set(t_clone.clone()); },
                                    "{label}"
                                }
                            }
                        }
                    }
                }
            }

            // ── Content ──
            div {
                style: "flex:1; overflow-y:auto; padding:16px;",
                match tab.read().clone() {
                    ReportTab::Expense    => rsx! { ExpenseReport {} },
                    ReportTab::Timeline   => rsx! { TimelineReport {} },
                    ReportTab::Investment => rsx! { InvestmentReport {} },
                }
            }
        }
    }
}
