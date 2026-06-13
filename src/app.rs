use dioxus::prelude::*;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

use crate::views::{
    dashboard::Dashboard,
    reports::Reports,
    settings::Settings,
    transactions::Transactions,
    wallets::Wallets,
};

// ─── AppState ─────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
}

impl AppState {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }

    pub fn with_conn<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Connection) -> T,
    {
        let conn = self.db.lock().expect("DB lock poisoned");
        f(&conn)
    }
}

// ─── Tabs ─────────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
pub enum Tab {
    Dashboard,
    Transactions,
    Wallets,
    Reports,
    Settings,
}

// ─── Root App ─────────────────────────────────────────────────────────────────

#[component]
pub fn App() -> Element {
    let db = crate::db::get();
    let state = AppState::new(db);
    provide_context(state);

    let active_tab = use_signal(|| Tab::Dashboard);

    rsx! {
        div {
            style: "display:flex; flex-direction:column; height:100vh; font-family:system-ui,sans-serif; background:#f5f5f5;",
            div {
                style: "flex:1; overflow-y:auto;",
                match *active_tab.read() {
                    Tab::Dashboard    => rsx! { Dashboard {} },
                    Tab::Transactions => rsx! { Transactions {} },
                    Tab::Wallets      => rsx! { Wallets {} },
                    Tab::Reports      => rsx! { Reports {} },
                    Tab::Settings     => rsx! { Settings {} },
                }
            }
            BottomTabBar { active_tab }
        }
    }
}

// ─── BottomTabBar ─────────────────────────────────────────────────────────────

#[component]
fn BottomTabBar(active_tab: Signal<Tab>) -> Element {
    let tabs: &[(Tab, &str, &str)] = &[
        (Tab::Dashboard,    "🏠", "Overview"),
        (Tab::Transactions, "💳", "Txns"),
        (Tab::Wallets,      "👛", "Wallets"),
        (Tab::Reports,      "📊", "Reports"),
        (Tab::Settings,     "⚙️",  "Settings"),
    ];

    rsx! {
        div {
            style: "display:flex; background:#fff; border-top:1px solid #e0e0e0; padding:8px 0 12px;",
            for (tab, icon, label) in tabs.iter().cloned() {
                {
                    let is_active = *active_tab.read() == tab;
                    let tab_clone = tab.clone();
                    rsx! {
                        button {
                            style: "flex:1; border:none; background:transparent; cursor:pointer; display:flex; flex-direction:column; align-items:center; gap:2px; padding:4px 0;",
                            onclick: move |_| { active_tab.set(tab_clone.clone()); },
                            span { style: "font-size:22px; line-height:1;", "{icon}" }
                            span {
                                style: if is_active { "font-size:10px; font-weight:700; color:#6366f1;" }
                                       else { "font-size:10px; color:#9ca3af;" },
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
    }
}