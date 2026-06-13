use dioxus::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::app::AppState;
use crate::models::{Category, NewCategory};
use crate::repository::CategoryRepo;

#[derive(Clone, PartialEq)]
enum CatMode { List, Add, Edit(Category) }

#[component]
pub fn CategoriesSettings() -> Element {
    let state = use_context::<AppState>();
    let mode  = use_signal(|| CatMode::List);
    let cats  = use_signal(|| vec![]);

    let load = {
        let state = state.clone();
        let mut cats = cats.clone();
        move || {
            state.with_conn(|conn| {
                if let Ok(list) = CategoryRepo::list(conn) { cats.set(list); }
            });
        }
    };

    use_effect({
        let mut load = load.clone();
        move || {
            load();
        }
    });

    let current_mode = mode.read().clone();
    match current_mode {
        CatMode::List => rsx! { CatList { cats: cats.read().clone(), mode, on_reload: load } },
        CatMode::Add  => rsx! { CatForm { mode, editing: None, on_reload: load } },
        CatMode::Edit(cat) => rsx! { CatForm { mode, editing: Some(cat), on_reload: load } },
    }
}

// ─── List ─────────────────────────────────────────────────────────────────────

#[component]
fn CatList(
    cats: Vec<Category>,
    mode: Signal<CatMode>,
    on_reload: EventHandler<()>,
) -> Element {
    let state   = use_context::<AppState>();
    let mut confirm = use_signal(|| None::<String>);

    let user_cats:   Vec<&Category> = cats.iter().filter(|c| !c.is_system).collect();
    let system_cats: Vec<&Category> = cats.iter().filter(|c|  c.is_system).collect();

    rsx! {
        div {
            style: "display:flex; flex-direction:column; gap:14px;",

            // ── Header ──
            div {
                style: "display:flex; justify-content:space-between; align-items:center;",
                h2 { style: "margin:0; font-size:16px; font-weight:700; color:#1f2937;", "Categories" }
                button {
                    style: "background:#6366f1; color:#fff; border:none; border-radius:20px; padding:8px 16px; font-size:14px; font-weight:600; cursor:pointer;",
                    onclick: move |_| { mode.set(CatMode::Add); },
                    "+ Add"
                }
            }

            // ── User categories ──
            if !user_cats.is_empty() {
                div {
                    style: "background:#fff; border-radius:12px; overflow:hidden; box-shadow:0 1px 3px rgba(0,0,0,.06);",
                    SectionLabel { text: "Custom" }
                    for cat in user_cats.iter() {
                        CatRow {
                            cat: (*cat).clone(),
                            on_edit: {
                                let c = (*cat).clone();
                                move |_| { mode.set(CatMode::Edit(c.clone())); }
                            },
                            on_delete: {
                                let id = cat.id.clone();
                                move |_| { confirm.set(Some(id.clone())); }
                            }
                        }
                    }
                }
            }

            // ── System categories (read-only) ──
            if !system_cats.is_empty() {
                div {
                    style: "background:#fff; border-radius:12px; overflow:hidden; box-shadow:0 1px 3px rgba(0,0,0,.06);",
                    SectionLabel { text: "System (read-only)" }
                    for cat in system_cats.iter() {
                        CatRow {
                            cat: (*cat).clone(),
                            on_edit: move |_| {},
                            on_delete: move |_| {}
                        }
                    }
                }
            }

            // ── Delete confirm ──
            if let Some(cat_id) = confirm.read().clone() {
                div {
                    style: "position:fixed; inset:0; background:rgba(0,0,0,.4); display:flex; align-items:center; justify-content:center; z-index:100;",
                    div {
                        style: "background:#fff; border-radius:16px; padding:24px; margin:16px; max-width:320px; width:100%;",
                        h3 { style: "margin:0 0 8px; font-size:17px;", "Delete category?" }
                        p  { style: "margin:0 0 20px; font-size:14px; color:#6b7280;", "Categories in use by transactions cannot be deleted." }
                        div { style: "display:flex; gap:10px;",
                            button {
                                style: "flex:1; padding:10px; border:1px solid #e5e7eb; border-radius:8px; background:#fff; cursor:pointer; font-size:14px;",
                                onclick: move |_| { confirm.set(None); },
                                "Cancel"
                            }
                            button {
                                style: "flex:1; padding:10px; border:none; border-radius:8px; background:#ef4444; color:#fff; cursor:pointer; font-size:14px; font-weight:600;",
                                onclick: {
                                    let state = state.clone();
                                    let cat_id = cat_id.clone();
                                    let on_reload = on_reload.clone();
                                    move |_| {
                                        state.with_conn(|conn| {
                                            let _ = CategoryRepo::delete(conn, &cat_id);
                                        });
                                        confirm.set(None);
                                        on_reload.call(());
                                    }
                                },
                                "Delete"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ─── Form ─────────────────────────────────────────────────────────────────────

#[component]
fn CatForm(
    mode: Signal<CatMode>,
    editing: Option<Category>,
    on_reload: EventHandler<()>,
) -> Element {
    let state = use_context::<AppState>();

    let mut name          = use_signal(|| editing.as_ref().map(|c| c.name.clone()).unwrap_or_default());
    let mut icon          = use_signal(|| editing.as_ref().and_then(|c| c.icon.clone()).unwrap_or_default());
    let mut color         = use_signal(|| editing.as_ref().and_then(|c| c.color.clone()).unwrap_or_else(|| "#6366f1".into()));
    let mut budget_str    = use_signal(|| editing.as_ref().and_then(|c| c.budget_amount.map(|b| format!("{b}"))).unwrap_or_default());
    let error_msg     = use_signal(|| String::new());

    let title = if editing.is_some() { "Edit Category" } else { "New Category" };

    let preset_icons = ["🍜","🚗","🛍️","💊","🎮","📚","🏠","💆","✈️","📦","☕","🎵","🐶","💡","🏋️"];

    let submit = {
        let state = state.clone();
        let name = name.clone();
        let icon = icon.clone();
        let color = color.clone();
        let budget_str = budget_str.clone();
        let mut error_msg = error_msg.clone();
        let editing = editing.clone();
        let on_reload = on_reload.clone();

        move |_: MouseEvent| {
            let name_val = name.read().trim().to_string();
            if name_val.is_empty() { error_msg.set("Name required".into()); return; }

            let budget = if budget_str.read().is_empty() {
                None
            } else {
                match Decimal::from_str(&budget_str.read()) {
                    Ok(b) => Some(b),
                    Err(_) => { error_msg.set("Invalid budget amount".into()); return; }
                }
            };

            let icon_opt  = { let i = icon.read();  if i.is_empty() { None } else { Some(i.clone()) } };
            let color_opt = { let c = color.read(); if c.is_empty() { None } else { Some(c.clone()) } };

            let result = state.with_conn(|conn| {
                if let Some(ref old) = editing {
                    let mut updated = old.clone();
                    updated.name          = name_val.clone();
                    updated.icon          = icon_opt;
                    updated.color         = color_opt;
                    updated.budget_amount = budget;
                    CategoryRepo::update(conn, &updated)
                } else {
                    CategoryRepo::create(conn, &NewCategory {
                        name: name_val,
                        icon: icon_opt,
                        color: color_opt,
                        budget_amount: budget,
                        parent_id: None,
                        sort_order: 0,
                    }).map(|_| ())
                }
            });

            match result {
                Ok(_)  => { mode.set(CatMode::List); on_reload.call(()); }
                Err(e) => { error_msg.set(e.to_string()); }
            }
        }
    };

    rsx! {
        div {
            style: "display:flex; flex-direction:column; gap:14px;",

            div {
                style: "display:flex; align-items:center; gap:12px;",
                button {
                    style: "border:none; background:transparent; font-size:22px; cursor:pointer; padding:0;",
                    onclick: move |_| { mode.set(CatMode::List); },
                    "←"
                }
                h1 { style: "margin:0; font-size:18px; font-weight:700; color:#1f2937;", "{title}" }
            }

            FieldLabel { label: "Name" }
            input {
                style: "border:1px solid #e5e7eb; border-radius:8px; padding:10px 12px; font-size:16px; outline:none; background:#fff;",
                r#type: "text",
                placeholder: "e.g. Food & Drinks",
                value: "{name.read()}",
                oninput: move |e| { name.set(e.value()); },
            }

            FieldLabel { label: "Icon" }
            div {
                style: "display:flex; flex-wrap:wrap; gap:8px;",
                for emoji in preset_icons.iter() {
                    {
                        let e = emoji.to_string();
                        let is_active = *icon.read() == *emoji;
                        rsx! {
                            button {
                                style: if is_active {
                                    "width:40px; height:40px; border:2px solid #6366f1; border-radius:8px; background:#eef2ff; font-size:20px; cursor:pointer;"
                                } else {
                                    "width:40px; height:40px; border:1px solid #e5e7eb; border-radius:8px; background:#fff; font-size:20px; cursor:pointer;"
                                },
                                onclick: move |_| { icon.set(e.clone()); },
                                "{emoji}"
                            }
                        }
                    }
                }
            }

            FieldLabel { label: "Color" }
            div {
                style: "display:flex; flex-wrap:wrap; gap:8px;",
                for hex in ["#6366f1","#10b981","#ef4444","#f59e0b","#8b5cf6","#ec4899","#06b6d4","#84cc16","#f97316","#6b7280"] {
                    {
                        let h = hex.to_string();
                        let is_active = *color.read() == hex;
                        let style = format!(
                            "width:32px;height:32px;border-radius:50%;background:{};border:{};cursor:pointer;",
                            hex,
                            if is_active {
                                "3px solid #1f2937"
                            } else {
                                "2px solid transparent"
                            }
                        );
                        rsx! {
                            button {
                                style: "{style}",
                                onclick: move |_| { color.set(h.clone()); },
                            }
                        }
                    }
                }
            }

            FieldLabel { label: "Monthly Budget (optional)" }
            input {
                style: "border:1px solid #e5e7eb; border-radius:8px; padding:10px 12px; font-size:16px; outline:none; background:#fff;",
                r#type: "number",
                placeholder: "Leave empty for unlimited",
                value: "{budget_str.read()}",
                oninput: move |e| { budget_str.set(e.value()); },
            }

            if !error_msg.read().is_empty() {
                div {
                    style: "background:#fef2f2; border-radius:8px; padding:10px; font-size:13px; color:#ef4444;",
                    "{error_msg.read()}"
                }
            }

            button {
                style: "padding:14px; background:#6366f1; color:#fff; border:none; border-radius:12px; font-size:16px; font-weight:700; cursor:pointer;",
                onclick: submit,
                if editing.is_some() { "Update" } else { "Create Category" }
            }
        }
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

#[component]
fn CatRow(
    cat: Category,
    on_edit:   EventHandler<MouseEvent>,
    on_delete: EventHandler<MouseEvent>,
) -> Element {
    let icon  = cat.icon.as_deref().unwrap_or("📦");
    let color = cat.color.as_deref().unwrap_or("#6b7280");
    let budget_label = cat.budget_amount
        .map(|b| format!("Budget: {b:.0} ₫"))
        .unwrap_or_else(|| "No budget".into());

    rsx! {
        div {
            style: "display:flex; align-items:center; gap:12px; padding:12px 16px; border-bottom:1px solid #f9fafb;",
            div {
                style: "width:36px; height:36px; border-radius:8px; background:{color}22; display:flex; align-items:center; justify-content:center; font-size:18px; flex-shrink:0;",
                "{icon}"
            }
            div { style: "flex:1;",
                div { style: "font-size:14px; font-weight:600; color:#374151;", "{cat.name}" }
                div { style: "font-size:11px; color:#9ca3af;", "{budget_label}" }
            }
            if !cat.is_system {
                button {
                    style: "border:none; background:transparent; font-size:18px; cursor:pointer; padding:4px;",
                    onclick: move |e| { on_edit.call(e); },
                    "✏️"
                }
                button {
                    style: "border:none; background:transparent; font-size:18px; cursor:pointer; padding:4px;",
                    onclick: move |e| { on_delete.call(e); },
                    "🗑"
                }
            }
        }
    }
}

#[component]
fn SectionLabel(text: &'static str) -> Element {
    rsx! {
        div {
            style: "padding:8px 16px; background:#f9fafb; font-size:11px; font-weight:700; color:#9ca3af; text-transform:uppercase; letter-spacing:.5px;",
            "{text}"
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
