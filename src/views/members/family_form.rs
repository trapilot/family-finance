use dioxus::prelude::*;
use crate::app::AppState;
use crate::models::Family;
use crate::repository::FamilyRepo;
use super::MemberView;

#[component]
pub fn FamilyForm(view: Signal<MemberView>, editing: Option<Family>) -> Element {
    let state = use_context::<AppState>();
    let is_editing = editing.is_some();
    let title = if is_editing { "Edit Family" } else { "Add Family" };

    let mut name = use_signal(|| editing.as_ref().map(|f| f.name.clone()).unwrap_or_default());
    let mut common_address = use_signal(|| editing.as_ref().and_then(|f| f.common_address.clone()).unwrap_or_default());
    let mut note = use_signal(|| editing.as_ref().and_then(|f| f.note.clone()).unwrap_or_default());

    let state1 = state.clone();
    let state2 = state.clone();

    rsx! {
        div {
            style: "padding:16px; display:flex; flex-direction:column; gap:14px;",

            // Header
            div {
                style: "display:flex; align-items:center; gap:12px;",
                button {
                    style: "background:transparent; border:none; font-size:20px; cursor:pointer; color:#6b7280;",
                    onclick: move |_| { view.set(MemberView::List); },
                    "‹"
                }
                h1 { style: "margin:0; font-size:20px; font-weight:700; color:#1f2937;", "{title}" }
            }

            // Name input
            div {
                style: "display:flex; flex-direction:column; gap:6px;",
                label { style: "font-size:14px; font-weight:600; color:#374151;", "Family Name" }
                input {
                    value: "{name.read()}",
                    style: "padding:10px 12px; border:1px solid #d1d5db; border-radius:10px; font-size:16px;",
                    oninput: move |e| name.set(e.value()),
                }
            }

            // Common address
            div {
                style: "display:flex; flex-direction:column; gap:6px;",
                label { style: "font-size:14px; font-weight:600; color:#374151;", "Common Address" }
                textarea {
                    value: "{common_address.read()}",
                    style: "padding:10px 12px; border:1px solid #d1d5db; border-radius:10px; font-size:16px; min-height:80px; resize:vertical;",
                    oninput: move |e| common_address.set(e.value()),
                }
            }

            // Note
            div {
                style: "display:flex; flex-direction:column; gap:6px;",
                label { style: "font-size:14px; font-weight:600; color:#374151;", "Note" }
                textarea {
                    value: "{note.read()}",
                    style: "padding:10px 12px; border:1px solid #d1d5db; border-radius:10px; font-size:16px; min-height:80px; resize:vertical;",
                    oninput: move |e| note.set(e.value()),
                }
            }

            // Save button
            button {
                style: "background:#6366f1; color:#fff; border:none; border-radius:14px; padding:14px; font-size:16px; font-weight:600; cursor:pointer; margin-top:8px;",
                onclick: move |_| {
                    let name_val = name.read().trim().to_string();
                    if name_val.is_empty() {
                        return;
                    }

                    state1.with_conn(|conn| {
                        if let Some(family) = &editing {
                            let mut updated = family.clone();
                            updated.name = name_val;
                            updated.common_address = if common_address.read().is_empty() { None } else { Some(common_address.read().clone()) };
                            updated.note = if note.read().is_empty() { None } else { Some(note.read().clone()) };
                            let _ = FamilyRepo::update(conn, &updated);
                        } else {
                            let new_family = crate::models::family::NewFamily {
                                name: name_val,
                                common_address: if common_address.read().is_empty() { None } else { Some(common_address.read().clone()) },
                                note: if note.read().is_empty() { None } else { Some(note.read().clone()) },
                            };
                            let _ = FamilyRepo::create(conn, &new_family);
                        }
                    });
                    view.set(MemberView::List);
                },
                "Save"
            }

            // Delete button (only for editing)
            if is_editing {
                button {
                    style: "background:#ef4444; color:#fff; border:none; border-radius:14px; padding:14px; font-size:16px; font-weight:600; cursor:pointer;",
                    onclick: {
                        let family = editing.clone().unwrap();
                        move |_| {
                            state2.with_conn(|conn| {
                                let _ = FamilyRepo::delete(conn, &family.id);
                            });
                            view.set(MemberView::List);
                        }
                    },
                    "Delete Family"
                }
            }
        }
    }
}
