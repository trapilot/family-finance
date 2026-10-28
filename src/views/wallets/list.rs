use dioxus::prelude::*;
use crate::i18n::use_i18n;

use crate::app::AppState;
use crate::models::{Currency, Wallet, WalletType};
use crate::repository::WalletRepo;
use crate::views::{fmt_currency, fmt_gold, fmt_usd, fmt_vnd};
use super::WalletView;

#[component]
pub fn WalletList(view: Signal<WalletView>) -> Element {
    let t = use_i18n();
    let state = use_context::<AppState>();
    let wallets = use_signal(|| vec![]);

    use_effect({
        let state = state.clone();
        let mut wallets = wallets.clone();
        move || {
            state.with_conn(|conn| {
                if let Ok(ws) = WalletRepo::list_active(conn) { wallets.set(ws); }
            });
        }
    });

    // Group wallets by type, in display order (empty groups are skipped)
    let groups: Vec<(&'static str, Vec<Wallet>)> = [
        (WalletType::Cash,       "Cash"),
        (WalletType::Bank,       "Bank"),
        (WalletType::EWallet,    "E-Wallet"),
        (WalletType::Investment, "Investment"),
        (WalletType::Loan,       "Loan"),
    ]
    .into_iter()
    .map(|(wallet_type, title)| {
        let ws: Vec<Wallet> = wallets.read().iter()
            .filter(|w| w.wallet_type == wallet_type)
            .cloned().collect();
        (title, ws)
    })
    .filter(|(_, ws)| !ws.is_empty())
    .collect();

    // Totals exclude investment wallets (their value lives in holdings)
    let regular: Vec<Wallet> = wallets.read().iter()
        .filter(|w| w.wallet_type != WalletType::Investment)
        .cloned().collect();

    // Net worth per currency
    let total_vnd:  rust_decimal::Decimal = regular.iter().filter(|w| w.currency == Currency::VND).map(|w| w.balance).sum();
    let total_usd:  rust_decimal::Decimal = regular.iter().filter(|w| w.currency == Currency::USD).map(|w| w.balance).sum();
    let total_gold: rust_decimal::Decimal = regular.iter().filter(|w| w.currency == Currency::GOLD).map(|w| w.balance).sum();

    rsx! {
        div {
            style: "padding:16px; display:flex; flex-direction:column; gap:16px;",

            // ── Header ──
            div {
                style: "display:flex; justify-content:space-between; align-items:center;",
                h1 { style: "margin:0; font-size:20px; font-weight:700; color:#1f2937;", {t.t("Wallets")} }
                button {
                    style: "background:#6366f1; color:#fff; border:none; border-radius:20px; padding:8px 16px; font-size:14px; font-weight:600; cursor:pointer;",
                    onclick: move |_| { view.set(WalletView::Add); },
                    {t.t("+ Add")}
                }
            }

            // ── Totals ──
            div {
                style: "display:grid; grid-template-columns:repeat(3,1fr); gap:8px;",
                TotalCard { label: "VND",  value: fmt_vnd(total_vnd),   color: "#6366f1" }
                TotalCard { label: "USD",  value: fmt_usd(total_usd),   color: "#10b981" }
                TotalCard { label: "GOLD", value: fmt_gold(total_gold), color: "#f59e0b" }
            }

            // ── Wallets grouped by type: Cash > Bank > E-Wallet > Investment > Loan ──
            for (title, group) in groups.iter() {
                SectionHeader { title: t.t(title) }
                for wallet in group.iter() {
                    WalletCard {
                        wallet: wallet.clone(),
                        on_tap: {
                            let w = wallet.clone();
                            move |_| { view.set(WalletView::Detail(w.clone())); }
                        },
                        on_edit: {
                            let w = wallet.clone();
                            move |_| { view.set(WalletView::Edit(w.clone())); }
                        }
                    }
                }
            }

            if wallets.read().is_empty() {
                div {
                    style: "text-align:center; padding:40px 0; color:#9ca3af;",
                    p { style: "font-size:32px; margin:0;", "👛" }
                    p { style: "font-size:14px; margin:8px 0 0;", {t.t("No wallets yet. Add one!")} }
                }
            }
        }
    }
}

// ─── Sub-components ───────────────────────────────────────────────────────────

#[component]
fn TotalCard(label: &'static str, value: String, color: &'static str) -> Element {
    rsx! {
        div {
            style: "background:#fff; border-radius:10px; padding:10px; box-shadow:0 1px 3px rgba(0,0,0,.06); border-left:3px solid {color};",
            div { style: "font-size:10px; font-weight:600; color:#9ca3af; text-transform:uppercase;", "{label}" }
            div { style: "font-size:12px; font-weight:700; color:#1f2937; margin-top:2px; word-break:break-all;", "{value}" }
        }
    }
}

#[component]
fn SectionHeader(title: &'static str) -> Element {
    rsx! {
        h2 { style: "margin:0; font-size:13px; font-weight:700; color:#6b7280; text-transform:uppercase; letter-spacing:.5px;", "{title}" }
    }
}

#[component]
fn WalletCard(
    wallet: Wallet,
    on_tap:  EventHandler<MouseEvent>,
    on_edit: EventHandler<MouseEvent>,
) -> Element {
    let t = use_i18n();
    let (type_icon, type_color) = match wallet.wallet_type {
        WalletType::Cash       => ("💵", "#10b981"),
        WalletType::Bank       => ("🏦", "#6366f1"),
        WalletType::Loan       => ("🤝", "#bb3baaff"),
        WalletType::EWallet    => ("📱", "#f59e0b"),
        WalletType::Investment => ("📈", "#8b5cf6"),
    };
    let balance_str = fmt_currency(wallet.balance, &wallet.currency);
    let icon = wallet.icon.as_deref().unwrap_or(type_icon);

    rsx! {
        div {
            style: "background:#fff; border-radius:12px; padding:14px; box-shadow:0 1px 3px rgba(0,0,0,.06); display:flex; align-items:center; gap:12px;",
            onclick: move |e| { on_tap.call(e); },

            div {
                style: "width:42px; height:42px; border-radius:10px; background:{type_color}22; display:flex; align-items:center; justify-content:center; font-size:22px; flex-shrink:0;",
                "{icon}"
            }

            div { style: "flex:1; min-width:0;",
                div { style: "font-size:15px; font-weight:600; color:#1f2937; overflow-wrap:anywhere;", "{wallet.name}" }
                div { style: "font-size:11px; color:#9ca3af; margin-top:1px;",
                    "{t.code(&wallet.wallet_type.to_string())} · {wallet.currency}"
                    if let Some(ref broker) = wallet.broker {
                        span { " · {broker}" }
                    }
                }
                // Balance gets its own line so long amounts never squeeze the name
                div { style: "font-size:16px; font-weight:700; color:#1f2937; margin-top:4px; overflow-wrap:anywhere;", "{balance_str}" }
            }

            button {
                style: "border:none; background:transparent; font-size:18px; cursor:pointer; padding:4px; flex-shrink:0;",
                onclick: move |e| { e.stop_propagation(); on_edit.call(e); },
                "✏️"
            }
        }
    }
}
