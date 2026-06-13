use dioxus::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::app::AppState;
use crate::models::{Currency, NewWallet, Wallet, WalletType};
use crate::repository::WalletRepo;
use super::WalletView;

#[component]
pub fn WalletForm(view: Signal<WalletView>, editing: Option<Wallet>) -> Element {
    let state = use_context::<AppState>();

    let mut name        = use_signal(|| editing.as_ref().map(|w| w.name.clone()).unwrap_or_default());
    let mut wallet_type = use_signal(|| editing.as_ref().map(|w| w.wallet_type.clone()).unwrap_or(WalletType::Cash));
    let mut currency    = use_signal(|| editing.as_ref().map(|w| w.currency.clone()).unwrap_or(Currency::VND));
    let mut balance_str = use_signal(|| editing.as_ref().map(|w| format!("{}", w.balance)).unwrap_or_else(|| "0".into()));
    let mut broker      = use_signal(|| editing.as_ref().and_then(|w| w.broker.clone()).unwrap_or_default());
    let mut icon        = use_signal(|| editing.as_ref().and_then(|w| w.icon.clone()).unwrap_or_default());
    let error_msg   = use_signal(|| String::new());

    let title = if editing.is_some() { "Edit Wallet" } else { "New Wallet" };

    let submit = {
        let state = state.clone();
        let name = name.clone();
        let wallet_type = wallet_type.clone();
        let currency = currency.clone();
        let balance_str = balance_str.clone();
        let broker = broker.clone();
        let icon = icon.clone();
        let mut error_msg = error_msg.clone();
        let editing = editing.clone();

        move |_: MouseEvent| {
            let name_val = name.read().trim().to_string();
            if name_val.is_empty() {
                error_msg.set("Name is required".into());
                return;
            }
            let balance = match Decimal::from_str(&balance_str.read()) {
                Ok(b) => b,
                Err(_) => { error_msg.set("Invalid balance".into()); return; }
            };
            let broker_opt = {
                let b = broker.read();
                if b.is_empty() { None } else { Some(b.clone()) }
            };
            let icon_opt = {
                let i = icon.read();
                if i.is_empty() { None } else { Some(i.clone()) }
            };

            let result = state.with_conn(|conn| {
                if let Some(ref old) = editing {
                    let mut updated = old.clone();
                    updated.name        = name_val.clone();
                    updated.wallet_type = wallet_type.read().clone();
                    updated.currency    = currency.read().clone();
                    updated.broker      = broker_opt.clone();
                    updated.icon        = icon_opt.clone();
                    WalletRepo::update(conn, &updated)
                } else {
                    WalletRepo::create(conn, &NewWallet {
                        name: name_val.clone(),
                        wallet_type: wallet_type.read().clone(),
                        currency: currency.read().clone(),
                        balance,
                        broker: broker_opt,
                        icon: icon_opt,
                        color: None,
                        sort_order: 0,
                    }).map(|_| ())
                }
            });

            match result {
                Ok(_)  => { view.set(WalletView::List); }
                Err(e) => { error_msg.set(e.to_string()); }
            }
        }
    };

    rsx! {
        div {
            style: "padding:16px; display:flex; flex-direction:column; gap:14px;",

            // ── Header ──
            div {
                style: "display:flex; align-items:center; gap:12px;",
                button {
                    style: "border:none; background:transparent; font-size:22px; cursor:pointer; padding:0;",
                    onclick: move |_| { view.set(WalletView::List); },
                    "←"
                }
                h1 { style: "margin:0; font-size:18px; font-weight:700; color:#1f2937;", "{title}" }
            }

            // ── Name ──
            FieldLabel { label: "Name" }
            input {
                style: "border:1px solid #e5e7eb; border-radius:8px; padding:10px 12px; font-size:16px; outline:none; background:#fff;",
                r#type: "text",
                placeholder: "e.g. Vietcombank",
                value: "{name.read()}",
                oninput: move |e| { name.set(e.value()); },
            }

            // ── Type ──
            FieldLabel { label: "Type" }
            div {
                style: "display:grid; grid-template-columns:1fr 1fr; gap:8px;",
                for (wt, label, icon_str) in [
                    (WalletType::Cash,       "Cash",       "💵"),
                    (WalletType::Bank,       "Bank",       "🏦"),
                    (WalletType::EWallet,    "E-Wallet",   "📱"),
                    (WalletType::Investment, "Investment", "📈"),
                ] {
                    {
                        let wt_clone = wt.clone();
                        let is_active = *wallet_type.read() == wt;
                        rsx! {
                            button {
                                style: if is_active {
                                    "padding:10px; border:2px solid #6366f1; border-radius:10px; background:#eef2ff; color:#6366f1; font-size:13px; font-weight:600; cursor:pointer; text-align:left;"
                                } else {
                                    "padding:10px; border:1px solid #e5e7eb; border-radius:10px; background:#fff; color:#6b7280; font-size:13px; cursor:pointer; text-align:left;"
                                },
                                onclick: move |_| { wallet_type.set(wt_clone.clone()); },
                                "{icon_str} {label}"
                            }
                        }
                    }
                }
            }

            // ── Currency ──
            FieldLabel { label: "Currency" }
            div {
                style: "display:flex; gap:8px;",
                for cur in [Currency::VND, Currency::USD, Currency::GOLD] {
                    {
                        let cur_clone = cur.clone();
                        let is_active = *currency.read() == cur;
                        rsx! {
                            button {
                                style: if is_active {
                                    "flex:1; padding:10px; border:none; border-radius:8px; background:#6366f1; color:#fff; font-size:14px; font-weight:700; cursor:pointer;"
                                } else {
                                    "flex:1; padding:10px; border:1px solid #e5e7eb; border-radius:8px; background:#fff; color:#6b7280; font-size:14px; cursor:pointer;"
                                },
                                onclick: move |_| { currency.set(cur_clone.clone()); },
                                "{cur}"
                            }
                        }
                    }
                }
            }

            // ── Initial balance (only for new) ──
            if editing.is_none() {
                FieldLabel { label: "Opening Balance" }
                input {
                    style: "border:1px solid #e5e7eb; border-radius:8px; padding:10px 12px; font-size:16px; outline:none; background:#fff;",
                    r#type: "number",
                    placeholder: "0",
                    value: "{balance_str.read()}",
                    oninput: move |e| { balance_str.set(e.value()); },
                }
            }

            // ── Broker (investment only) ──
            if *wallet_type.read() == WalletType::Investment {
                FieldLabel { label: "Broker" }
                input {
                    style: "border:1px solid #e5e7eb; border-radius:8px; padding:10px 12px; font-size:14px; outline:none; background:#fff;",
                    r#type: "text",
                    placeholder: "e.g. SSI, VPS, Binance",
                    value: "{broker.read()}",
                    oninput: move |e| { broker.set(e.value()); },
                }
            }

            // ── Icon ──
            FieldLabel { label: "Icon (emoji, optional)" }
            input {
                style: "border:1px solid #e5e7eb; border-radius:8px; padding:10px 12px; font-size:22px; outline:none; background:#fff; width:60px;",
                r#type: "text",
                placeholder: "🏦",
                value: "{icon.read()}",
                oninput: move |e| {
                    icon.set(e.value());
                },
            }

            // ── Error ──
            if !error_msg.read().is_empty() {
                div {
                    style: "background:#fef2f2; border-radius:8px; padding:10px 12px; font-size:13px; color:#ef4444;",
                    "{error_msg.read()}"
                }
            }

            // ── Submit ──
            button {
                style: "padding:14px; background:#6366f1; color:#fff; border:none; border-radius:12px; font-size:16px; font-weight:700; cursor:pointer; margin-top:4px;",
                onclick: submit,
                if editing.is_some() { "Update Wallet" } else { "Create Wallet" }
            }
        }
    }
}

#[component]
fn FieldLabel(label: &'static str) -> Element {
    rsx! {
        label {
            style: "font-size:12px; font-weight:600; color:#374151; text-transform:uppercase; letter-spacing:.5px;",
            "{label}"
        }
    }
}
