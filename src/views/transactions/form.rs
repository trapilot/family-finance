use dioxus::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::app::AppState;
use crate::models::{
    Currency, IncomeType, NewTransaction, Transaction, TransactionType,
};
use crate::repository::{CategoryRepo, HoldingRepo, WalletRepo};
use crate::services::TransactionService;
use crate::views::{date_str_to_ts, now_ts};
use super::TxnView;

#[component]
pub fn TransactionForm(view: Signal<TxnView>, editing: Option<Transaction>) -> Element {
    let state = use_context::<AppState>();

    // ── Reference data ──
    let wallets    = use_signal(|| vec![]);
    let categories = use_signal(|| vec![]);
    let holdings   = use_signal(|| vec![]);

    // ── Form fields ──
    let mut txn_type    = use_signal(|| editing.as_ref().map(|t| t.txn_type.clone()).unwrap_or(TransactionType::Expense));
    let mut wallet_id   = use_signal(|| editing.as_ref().map(|t| t.wallet_id.clone()).unwrap_or_default());
    let mut amount_str  = use_signal(|| editing.as_ref().map(|t| format!("{}", t.amount)).unwrap_or_default());
    let mut currency    = use_signal(|| editing.as_ref().map(|t| t.currency.clone()).unwrap_or(Currency::VND));
    let mut note        = use_signal(|| editing.as_ref().and_then(|t| t.note.clone()).unwrap_or_default());

    // date stored as "YYYY-MM-DD" for <input type=date>
    let mut txn_date = use_signal(|| {
        editing.as_ref().map(|t| {
            use chrono::{Local, TimeZone};
            Local.timestamp_opt(t.txn_date, 0)
                .single()
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default()
        }).unwrap_or_else(|| {
            use chrono::{Local, Datelike};
            let now = Local::now();
            format!("{}-{:02}-{:02}", now.year(), now.month(), now.day())
        })
    });

    // Income
    let mut income_type = use_signal(|| editing.as_ref().and_then(|t| t.income_type.clone()).unwrap_or(IncomeType::Salary));

    // Expense
    let mut category_id = use_signal(|| editing.as_ref().and_then(|t| t.category_id.clone()).unwrap_or_default());

    // Transfer
    let mut to_wallet_id  = use_signal(|| editing.as_ref().and_then(|t| t.to_wallet_id.clone()).unwrap_or_default());
    let mut to_amount_str = use_signal(|| editing.as_ref().and_then(|t| t.to_amount.map(|a| format!("{a}"))).unwrap_or_default());
    let mut to_currency   = use_signal(|| editing.as_ref().and_then(|t| t.to_currency.clone()).unwrap_or(Currency::VND));

    // Invest
    let mut holding_id = use_signal(|| editing.as_ref().and_then(|t| t.holding_id.clone()).unwrap_or_default());
    let mut qty_str    = use_signal(|| editing.as_ref().and_then(|t| t.asset_quantity.map(|q| format!("{q}"))).unwrap_or_default());
    let mut price_str  = use_signal(|| editing.as_ref().and_then(|t| t.asset_price.map(|p| format!("{p}"))).unwrap_or_default());

    let error_msg = use_signal(|| String::new());

    // ── Load reference data ──
    use_effect({
        let state = state.clone();
        let mut wallets    = wallets.clone();
        let mut categories = categories.clone();
        let mut holdings   = holdings.clone();
        move || {
            state.with_conn(|conn| {
                if let Ok(ws) = WalletRepo::list_active(conn)    { wallets.set(ws); }
                if let Ok(cs) = CategoryRepo::list(conn)          { categories.set(cs); }
                if let Ok(hs) = HoldingRepo::list_all_active(conn) { holdings.set(hs); }
            });
        }
    });

    // ── Submit ──
    let submit = {
        let state      = state.clone();
        let txn_type   = txn_type.clone();
        let wallet_id  = wallet_id.clone();
        let amount_str = amount_str.clone();
        let currency   = currency.clone();
        let note       = note.clone();
        let txn_date   = txn_date.clone();
        let income_type = income_type.clone();
        let category_id = category_id.clone();
        let to_wallet_id  = to_wallet_id.clone();
        let to_amount_str = to_amount_str.clone();
        let to_currency   = to_currency.clone();
        let holding_id = holding_id.clone();
        let qty_str    = qty_str.clone();
        let price_str  = price_str.clone();
        let mut error_msg = error_msg.clone();
        let editing = editing.clone();

        move |_: MouseEvent| {
            let amount = match Decimal::from_str(&amount_str.read()) {
                Ok(a) => a,
                Err(_) => { error_msg.set("Invalid amount".into()); return; }
            };
            let date_ts = match date_str_to_ts(&txn_date.read()) {
                Some(ts) => ts,
                None => now_ts(),
            };
            let note_opt = {
                let n = note.read();
                if n.is_empty() { None } else { Some(n.clone()) }
            };

            let mut new_txn = match txn_type.read().clone() {
                TransactionType::Income => {
                    NewTransaction::income(
                        &wallet_id.read(),
                        amount,
                        currency.read().clone(),
                        income_type.read().clone(),
                        date_ts,
                    )
                }
                TransactionType::Expense => {
                    NewTransaction::expense(
                        &wallet_id.read(),
                        amount,
                        currency.read().clone(),
                        &category_id.read(),
                        date_ts,
                    )
                }
                TransactionType::Transfer => {
                    let to_amount = match Decimal::from_str(&to_amount_str.read()) {
                        Ok(a) => a,
                        Err(_) => { error_msg.set("Invalid to_amount".into()); return; }
                    };
                    NewTransaction::transfer(
                        &wallet_id.read(),
                        amount,
                        currency.read().clone(),
                        &to_wallet_id.read(),
                        to_amount,
                        to_currency.read().clone(),
                        date_ts,
                    )
                }
                TransactionType::InvestBuy => {
                    let qty   = match Decimal::from_str(&qty_str.read())   { Ok(q) => q, Err(_) => { error_msg.set("Invalid quantity".into()); return; } };
                    let price = match Decimal::from_str(&price_str.read()) { Ok(p) => p, Err(_) => { error_msg.set("Invalid price".into()); return; } };
                    NewTransaction::invest_buy(&wallet_id.read(), &holding_id.read(), qty, price, currency.read().clone(), date_ts)
                }
                TransactionType::InvestSell => {
                    let qty   = match Decimal::from_str(&qty_str.read())   { Ok(q) => q, Err(_) => { error_msg.set("Invalid quantity".into()); return; } };
                    let price = match Decimal::from_str(&price_str.read()) { Ok(p) => p, Err(_) => { error_msg.set("Invalid price".into()); return; } };
                    NewTransaction::invest_sell(&wallet_id.read(), &holding_id.read(), qty, price, currency.read().clone(), date_ts)
                }
            };

            new_txn.note = note_opt;

            let result = state.with_conn(|conn| {
                // If editing: delete old then insert new (simple approach)
                if let Some(ref old) = editing {
                    let _ = TransactionService::delete(conn, &old.id);
                }
                TransactionService::execute(conn, &new_txn)
            });

            match result {
                Ok(_)  => { view.set(TxnView::List); }
                Err(e) => { error_msg.set(e.to_string()); }
            }
        }
    };

    let title = if editing.is_some() { "Edit Transaction" } else { "New Transaction" };

    rsx! {
        div {
            style: "padding:16px; display:flex; flex-direction:column; gap:14px;",

            // ── Header ──
            div {
                style: "display:flex; align-items:center; gap:12px;",
                button {
                    style: "border:none; background:transparent; font-size:22px; cursor:pointer; padding:0;",
                    onclick: move |_| { view.set(TxnView::List); },
                    "←"
                }
                h1 { style: "margin:0; font-size:18px; font-weight:700; color:#1f2937;", "{title}" }
            }

            // ── Transaction type selector ──
            div {
                style: "display:flex; gap:6px; flex-wrap:wrap;",
                for (tt, label) in [
                    (TransactionType::Expense,   "Expense"),
                    (TransactionType::Income,    "Income"),
                    (TransactionType::Transfer,  "Transfer"),
                    (TransactionType::InvestBuy, "Buy"),
                    (TransactionType::InvestSell,"Sell"),
                ] {
                    {
                        let tt_clone = tt.clone();
                        let is_active = *txn_type.read() == tt;
                        rsx! {
                            button {
                                style: if is_active {
                                    "padding:6px 14px; border:none; border-radius:20px; background:#6366f1; color:#fff; font-size:12px; font-weight:600; cursor:pointer;"
                                } else {
                                    "padding:6px 14px; border:1px solid #e5e7eb; border-radius:20px; background:#fff; color:#6b7280; font-size:12px; cursor:pointer;"
                                },
                                onclick: move |_| {
                                    txn_type.set(tt_clone.clone());
                                },
                                "{label}"
                            }
                        }
                    }
                }
            }

            // ── Wallet ──
            FieldLabel { label: "From Wallet" }
            SelectField {
                value: wallet_id.read().clone(),
                onchange: move |v| { wallet_id.set(v); },
                placeholder: "Select wallet",
                options: wallets.read().iter().map(|w| (w.id.clone(), format!("{} ({})", w.name, w.currency))).collect()
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
                                    "flex:1; padding:8px; border:none; border-radius:8px; background:#6366f1; color:#fff; font-size:13px; font-weight:600; cursor:pointer;"
                                } else {
                                    "flex:1; padding:8px; border:1px solid #e5e7eb; border-radius:8px; background:#fff; color:#6b7280; font-size:13px; cursor:pointer;"
                                },
                                onclick: move |_| { currency.set(cur_clone.clone()); },
                                "{cur}"
                            }
                        }
                    }
                }
            }

            // ── Amount ──
            FieldLabel { label: "Amount" }
            input {
                style: "border:1px solid #e5e7eb; border-radius:8px; padding:10px 12px; font-size:16px; outline:none; background:#fff;",
                r#type: "number",
                placeholder: "0",
                value: "{amount_str.read()}",
                oninput: move |e| {
                    amount_str.set(e.value());
                },
            }

            // ── Type-specific fields ──
            match txn_type.read().clone() {
                TransactionType::Income => rsx! {
                    FieldLabel { label: "Income Type" }
                    SelectField {
                        value: income_type.read().to_string(),
                        onchange: move |v: String| {
                            if let Ok(it) = IncomeType::from_str(&v) {
                                income_type.set(it);
                            }
                        },
                        placeholder: "Select type",
                        options: vec![
                            ("salary".into(),      "Salary".into()),
                            ("bonus".into(),       "Bonus".into()),
                            ("side_income".into(), "Side Income".into()),
                            ("rental".into(),      "Rental".into()),
                            ("investment".into(),  "Investment".into()),
                            ("other".into(),       "Other".into()),
                        ]
                    }
                },

                TransactionType::Expense => rsx! {
                    FieldLabel { label: "Category" }
                    SelectField {
                        value: category_id.read().clone(),
                        onchange: move |v| { category_id.set(v); },
                        placeholder: "Select category",
                        options: categories.read().iter().map(|c| (c.id.clone(), format!("{} {}", c.icon.as_deref().unwrap_or(""), c.name))).collect()
                    }
                },

                TransactionType::Transfer => rsx! {
                    FieldLabel { label: "To Wallet" }
                    SelectField {
                        value: to_wallet_id.read().clone(),
                        onchange: move |v| { to_wallet_id.set(v); },
                        placeholder: "Select destination wallet",
                        options: wallets.read().iter().map(|w| (w.id.clone(), format!("{} ({})", w.name, w.currency))).collect()
                    }
                    FieldLabel { label: "To Amount (leave same for same-currency)" }
                    input {
                        style: "border:1px solid #e5e7eb; border-radius:8px; padding:10px 12px; font-size:16px; outline:none; background:#fff;",
                        r#type: "number",
                        placeholder: "Received amount",
                        value: "{to_amount_str.read()}",
                        oninput: move |e| { to_amount_str.set(e.value()); },
                    }
                    FieldLabel { label: "To Currency" }
                    div {
                        style: "display:flex; gap:8px;",
                        for cur in [Currency::VND, Currency::USD, Currency::GOLD] {
                            {
                                let cur_clone = cur.clone();
                                let is_active = *to_currency.read() == cur;
                                rsx! {
                                    button {
                                        style: if is_active {
                                            "flex:1; padding:8px; border:none; border-radius:8px; background:#6366f1; color:#fff; font-size:13px; font-weight:600; cursor:pointer;"
                                        } else {
                                            "flex:1; padding:8px; border:1px solid #e5e7eb; border-radius:8px; background:#fff; color:#6b7280; font-size:13px; cursor:pointer;"
                                        },
                                        onclick: move |_| { to_currency.set(cur_clone.clone()); },
                                        "{cur}"
                                    }
                                }
                            }
                        }
                    }
                },

                TransactionType::InvestBuy | TransactionType::InvestSell => rsx! {
                    FieldLabel { label: "Holding" }
                    SelectField {
                        value: holding_id.read().clone(),
                        onchange: move |v| { holding_id.set(v); },
                        placeholder: "Select symbol",
                        options: holdings.read().iter().map(|h| (h.id.clone(), format!("{} ({})", h.symbol, h.asset_type))).collect()
                    }
                    FieldLabel { label: "Quantity" }
                    input {
                        style: "border:1px solid #e5e7eb; border-radius:8px; padding:10px 12px; font-size:16px; outline:none; background:#fff;",
                        r#type: "number",
                        placeholder: "0",
                        value: "{qty_str.read()}",
                        oninput: move |e| { qty_str.set(e.value()); },
                    }
                    FieldLabel { label: "Price per unit" }
                    input {
                        style: "border:1px solid #e5e7eb; border-radius:8px; padding:10px 12px; font-size:16px; outline:none; background:#fff;",
                        r#type: "number",
                        placeholder: "0",
                        value: "{price_str.read()}",
                        oninput: move |e| { price_str.set(e.value()); },
                    }
                },
            }

            // ── Date ──
            FieldLabel { label: "Date" }
            input {
                style: "border:1px solid #e5e7eb; border-radius:8px; padding:10px 12px; font-size:14px; outline:none; background:#fff;",
                r#type: "date",
                value: "{txn_date.read()}",
                oninput: move |e| { txn_date.set(e.value()); },
            }

            // ── Note ──
            FieldLabel { label: "Note (optional)" }
            input {
                style: "border:1px solid #e5e7eb; border-radius:8px; padding:10px 12px; font-size:14px; outline:none; background:#fff;",
                r#type: "text",
                placeholder: "Add a note…",
                value: "{note.read()}",
                oninput: move |e| { note.set(e.value()); },
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
                if editing.is_some() { "Update Transaction" } else { "Add Transaction" }
            }
        }
    }
}

// ─── Shared small components ─────────────────────────────────────────────────

#[component]
fn FieldLabel(label: &'static str) -> Element {
    rsx! {
        label {
            style: "font-size:12px; font-weight:600; color:#374151; text-transform:uppercase; letter-spacing:.5px;",
            "{label}"
        }
    }
}

#[component]
fn SelectField(
    value: String,
    onchange: EventHandler<String>,
    placeholder: &'static str,
    options: Vec<(String, String)>,
) -> Element {
    rsx! {
        select {
            style: "border:1px solid #e5e7eb; border-radius:8px; padding:10px 12px; font-size:14px; outline:none; background:#fff; width:100%; appearance:none;",
            value: "{value}",
            onchange: move |e| { onchange.call(e.value()); },
            option { value: "", disabled: true, selected: value.is_empty(), "{placeholder}" }
            for (val, label) in options.iter() {
                option { value: "{val}", selected: *val == value, "{label}" }
            }
        }
    }
}
