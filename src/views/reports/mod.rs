pub mod expense;
pub mod investment;
pub mod timeline;
pub mod type_transactions;

use dioxus::prelude::*;
use crate::i18n::use_i18n;
use expense::ExpenseReport;
use timeline::TimelineReport;
use investment::InvestmentReport;
use type_transactions::{TypeDrill, TypeTransactions};

#[derive(Clone, PartialEq)]
enum ReportTab { Expense, Timeline, Investment }

#[component]
pub fn Reports() -> Element {
    let t = use_i18n();
    let mut tab = use_signal(|| ReportTab::Expense);
    // Tapping Income / Expense in a report opens its transactions; reports read this via context
    let mut drill = use_signal(|| None::<TypeDrill>);
    use_context_provider(|| drill);

    rsx! {
        if let Some(d) = drill() {
            TypeTransactions { drill: d, on_back: move |_| drill.set(None) }
        }
        // Kept mounted while the list is open, so Back returns to the same month / period
        div {
            style: if drill.read().is_some() { "display:none;" } else { "display:flex; flex-direction:column; height:100%;" },

            // ── Header ──
            div {
                style: "padding:16px 16px 0; background:#fff;",
                h1 { style: "margin:0 0 12px; font-size:20px; font-weight:700; color:#1f2937;", {t.t("Reports")} }
                div {
                    style: "display:flex; gap:0; border-bottom:1px solid #f3f4f6;",
                    for (tab_kind, label) in [
                        (ReportTab::Expense,    t.t("Expense")),
                        (ReportTab::Timeline,   t.t("Timeline")),
                        (ReportTab::Investment, t.t("Invest")),
                    ] {
                        {
                            let is_active = *tab.read() == tab_kind;
                            let t_clone = tab_kind.clone();
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
