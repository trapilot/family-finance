pub mod backup;
pub mod categories;

use dioxus::prelude::*;
use categories::CategoriesSettings;
use backup::BackupSettings;

#[derive(Clone, PartialEq)]
enum SettingsTab { Categories, Backup }

#[component]
pub fn Settings() -> Element {
    let mut tab = use_signal(|| SettingsTab::Categories);

    rsx! {
        div {
            style: "display:flex; flex-direction:column; height:100%;",

            // ── Header ──
            div {
                style: "padding:16px 16px 0; background:#fff;",
                h1 { style: "margin:0 0 12px; font-size:20px; font-weight:700; color:#1f2937;", "Settings" }
                div {
                    style: "display:flex; border-bottom:1px solid #f3f4f6;",
                    for (t, label) in [(SettingsTab::Categories, "Categories"), (SettingsTab::Backup, "Backup")] {
                        {
                            let is_active = *tab.read() == t;
                            let t_clone = t.clone();
                            rsx! {
                                button {
                                    style: if is_active {
                                        "flex:1; padding:10px 0; border:none; border-bottom:2px solid #6366f1; background:#fff; font-size:14px; font-weight:600; color:#6366f1; cursor:pointer;"
                                    } else {
                                        "flex:1; padding:10px 0; border:none; border-bottom:2px solid transparent; background:#fff; font-size:14px; color:#9ca3af; cursor:pointer;"
                                    },
                                    onclick: move |_| { tab.set(t_clone.clone()); },
                                    "{label}"
                                }
                            }
                        }
                    }
                }
            }

            div {
                style: "flex:1; overflow-y:auto; padding:16px;",
                match tab.read().clone() {
                    SettingsTab::Categories => rsx! { CategoriesSettings {} },
                    SettingsTab::Backup     => rsx! { BackupSettings {} },
                }
            }
        }
    }
}
