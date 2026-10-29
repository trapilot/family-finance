use dioxus::prelude::*;
use crate::i18n::use_i18n;
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::app::AppState;
use crate::models::{Currency, NewTransaction, NewWallet, Wallet, WalletType};
use crate::repository::WalletRepo;
use crate::services::TransactionService;
use super::WalletView;
use crate::views::{now_ts, AmountInput};

#[component]
pub fn WalletForm(view: Signal<WalletView>, editing: Option<Wallet>) -> Element {
    let t = use_i18n();
    let state = use_context::<AppState>();

    let mut name        = use_signal(|| editing.as_ref().map(|w| w.name.clone()).unwrap_or_default());
    let mut wallet_type = use_signal(|| editing.as_ref().map(|w| w.wallet_type.clone()).unwrap_or(WalletType::Bank));
    let mut currency    = use_signal(|| editing.as_ref().map(|w| w.currency.clone()).unwrap_or(Currency::VND));
    // Opening balance: the wallet's "Opening Balance" transaction (editable later, 0 allowed)
    let mut balance_str = use_signal({
        let state = state.clone();
        let editing = editing.clone();
        move || {
            editing.as_ref()
                .and_then(|w| state.with_conn(|conn| TransactionService::find_opening_balance(conn, &w.id).ok().flatten()))
                .map(|t| t.amount.normalize().to_string())
                .unwrap_or_else(|| "0".into())
        }
    });
    let mut broker      = use_signal(|| editing.as_ref().and_then(|w| w.broker.clone()).unwrap_or_default());
    let mut icon        = use_signal(|| editing.as_ref().and_then(|w| w.icon.clone()).unwrap_or_default());
    let error_msg   = use_signal(|| String::new());
    // Editing only: whether the wallet is unused (see `WalletRepo::can_delete`), and the confirm dialog
    let can_delete = use_signal({
        let state = state.clone();
        let editing = editing.clone();
        move || editing.as_ref().is_some_and(|w| state.with_conn(|conn| WalletRepo::can_delete(conn, &w.id).unwrap_or(false)))
    });
    let mut confirm_delete = use_signal(|| false);

    let title = if editing.is_some() { t.t("Edit Wallet") } else { t.t("New Wallet") };

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
            let name_val = name.read().trim().to_uppercase();
            if name_val.is_empty() {
                error_msg.set(t.t("Name is required").into());
                return;
            }
            // Empty field = 0
            let balance = match balance_str.read().as_str() {
                "" => Decimal::ZERO,
                s  => match Decimal::from_str(s) {
                    Ok(b) => b,
                    Err(_) => { error_msg.set(t.t("Invalid balance").into()); return; }
                },
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
                    WalletRepo::update(conn, &updated)?;
                    TransactionService::set_opening_balance(conn, &updated.id, balance, updated.currency.clone(), t.t("Opening Balance"))
                } else {
                    // The balance only ever moves through transactions, so the opening amount
                    // is recorded as an "Opening Balance" transaction on a wallet created at 0.
                    let wallet = WalletRepo::create(conn, &NewWallet {
                        name: name_val.clone(),
                        wallet_type: wallet_type.read().clone(),
                        currency: currency.read().clone(),
                        balance: Decimal::ZERO,
                        broker: broker_opt,
                        icon: icon_opt,
                        color: None,
                        sort_order: 0,
                    })?;
                    // Recorded even when 0, so it can be changed later from Edit Wallet
                    let mut opening = NewTransaction::opening_balance(&wallet.id, balance, wallet.currency.clone(), now_ts());
                    opening.note = Some(t.t("Opening Balance").to_string());
                    TransactionService::execute(conn, &opening)?;
                    Ok(())
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

            // ── Type (first option = default for a new wallet) ──
            FieldLabel { label: t.t("Type") }
            select {
                style: "border:1px solid #e5e7eb; border-radius:8px; padding:10px 12px; font-size:16px; outline:none; background:#fff; width:100%; box-sizing:border-box;",
                oninput: move |e| {
                    if let Ok(wt) = e.value().parse::<WalletType>() { wallet_type.set(wt); }
                },
                for (wt, label, icon_str) in [
                    (WalletType::Bank,       t.t("Bank"),       "🏦"),
                    (WalletType::Cash,       t.t("Cash"),       "💵"),
                    (WalletType::EWallet,    t.t("E-Wallet"),   "📱"),
                    (WalletType::Investment, t.t("Investment"), "📈"),
                    (WalletType::Loan,       t.t("Loan"),       "🤝"),
                ] {
                    option {
                        value: "{wt}",
                        selected: *wallet_type.read() == wt,
                        "{icon_str} {label}"
                    }
                }
            }

            // ── Name ──
            FieldLabel { label: t.t("Name") }
            input {
                style: "border:1px solid #e5e7eb; border-radius:8px; padding:10px 12px; font-size:16px; outline:none; background:#fff;",
                r#type: "text",
                // Wallet names are UPPERCASE: caps keyboard on phones, and converted as you type
                autocapitalize: "characters",
                placeholder: t.t("e.g. HSBC"),
                value: "{name.read()}",
                oninput: move |e| { name.set(e.value().to_uppercase()); },
            }

            // ── Currency ──
            FieldLabel { label: t.t("Currency") }
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

            // ── Opening balance (new and edit) ──
            FieldLabel { label: t.t("Opening Balance") }
            AmountInput {
                value: balance_str.read().clone(),
                oninput: move |v| balance_str.set(v),
            }

            // ── Broker (investment only) ──
            if *wallet_type.read() == WalletType::Investment {
                FieldLabel { label: t.t("Broker") }
                input {
                    style: "border:1px solid #e5e7eb; border-radius:8px; padding:10px 12px; font-size:14px; outline:none; background:#fff;",
                    r#type: "text",
                    placeholder: t.t("e.g. HSBC, VPS, Binance, OKX"),
                    value: "{broker.read()}",
                    oninput: move |e| { broker.set(e.value()); },
                }
            }

            // ── Icon ──
            FieldLabel { label: t.t("Icon (emoji, optional)") }
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
                if editing.is_some() { {t.t("Update Wallet")} } else { {t.t("Create Wallet")} }
            }

            // ── Delete (unused wallets only) ──
            if let Some(wallet) = editing.clone() {
                if can_delete() {
                    button {
                        style: "padding:12px; background:transparent; color:#ef4444; border:1px solid #ef4444; border-radius:12px; font-size:15px; font-weight:600; cursor:pointer;",
                        onclick: move |_| confirm_delete.set(true),
                        {t.t("🗑 Delete wallet")}
                    }
                } else {
                    div { style: "font-size:12px; color:#9ca3af; text-align:center;",
                        {t.t("This wallet has transactions, so it can't be deleted.")}
                    }
                }

                if confirm_delete() {
                    div {
                        style: "position:fixed; inset:0; background:rgba(0,0,0,.45); display:flex; align-items:center; justify-content:center; z-index:100; padding:20px;",
                        div {
                            style: "background:#fff; border-radius:16px; padding:22px; max-width:320px; width:100%;",
                            h3 { style: "margin:0 0 8px; font-size:17px; font-weight:700;", {t.t("Delete wallet?")} }
                            p { style: "margin:0 0 18px; font-size:14px; color:#6b7280; line-height:1.5;",
                                {t.t("\"{name}\" will be permanently deleted.").replace("{name}", &wallet.name)}
                            }
                            div { style: "display:flex; gap:10px;",
                                button {
                                    style: "flex:1; padding:11px; border:1px solid #e5e7eb; border-radius:8px; background:#fff; font-size:14px; cursor:pointer;",
                                    onclick: move |_| confirm_delete.set(false),
                                    {t.t("Cancel")}
                                }
                                button {
                                    style: "flex:1; padding:11px; border:none; border-radius:8px; background:#ef4444; color:#fff; font-size:14px; font-weight:700; cursor:pointer;",
                                    onclick: {
                                        let state = state.clone();
                                        let mut error_msg = error_msg.clone();
                                        let id = wallet.id.clone();
                                        move |_| {
                                            confirm_delete.set(false);
                                            match state.with_conn(|conn| WalletRepo::delete(conn, &id)) {
                                                Ok(())  => view.set(WalletView::List),
                                                Err(e)  => error_msg.set(e.to_string()),
                                            }
                                        }
                                    },
                                    {t.t("Delete")}
                                }
                            }
                        }
                    }
                }
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
