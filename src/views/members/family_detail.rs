use dioxus::prelude::*;
use crate::app::AppState;
use crate::models::{Family, Member};
use crate::repository::MemberRepo;
use super::MemberView;

#[component]
pub fn FamilyDetail(view: Signal<MemberView>, family: Family) -> Element {
    let state = use_context::<AppState>();
    let members = use_signal(|| vec![]);

    let mut members_effect = members.clone();
    let family_id = family.id.clone();
    let state_effect = state.clone();

    use_effect(move || {
        state_effect.with_conn(|conn| {
            if let Ok(list) = MemberRepo::list_by_family(conn, &family_id) {
                members_effect.set(list);
            }
        });
    });

    let common_address_clone = family.common_address.clone();

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
                h1 { style: "margin:0; font-size:20px; font-weight:700; color:#1f2937;", "{family.name}" }
                button {
                    style: "background:#f3f4f6; color:#374151; border:none; border-radius:12px; padding:8px 12px; font-size:14px; font-weight:600; cursor:pointer; margin-left:auto;",
                    onclick: {
                        let family = family.clone();
                        move |_| { view.set(MemberView::EditFamily(family.clone())); }
                    },
                    "Edit"
                }
            }

            // Common address
            if let Some(addr) = common_address_clone {
                div {
                    style: "background:#fff; border-radius:14px; padding:14px; box-shadow:0 1px 4px rgba(0,0,0,.07);",
                    div { style: "display:flex; align-items:center; justify-content:space-between; gap:8px;",
                        div { style: "display:flex; flex-direction:column; gap:4px;",
                            div { style: "font-size:12px; font-weight:600; color:#6b7280;", "Common Address" }
                            div { style: "font-size:14px; color:#1f2937;", "{addr}" }
                        }
                        button {
                            style: "background:#eef2ff; color:#6366f1; border:none; border-radius:8px; padding:8px 12px; font-size:12px; font-weight:600; cursor:pointer;",
                            onclick: {
                                let addr = addr.clone();
                                move |_| {
                                    let js = format!("navigator.clipboard.writeText('{addr}').catch(()=>{{}})");
                                    let _ = dioxus::document::eval(&js);
                                }
                            },
                            "Copy"
                        }
                    }
                }
            }

            // Note
            if let Some(note) = &family.note {
                div {
                    style: "background:#fff; border-radius:14px; padding:14px; box-shadow:0 1px 4px rgba(0,0,0,.07);",
                    div { style: "font-size:12px; font-weight:600; color:#6b7280; margin-bottom:6px;", "Note" }
                    div { style: "font-size:14px; color:#1f2937;", "{note}" }
                }
            }

            // Members section
            if !members.read().is_empty() {
                div {
                    style: "display:flex; flex-direction:column; gap:8px;",
                    div { style: "font-size:11px; font-weight:700; color:#9ca3af; text-transform:uppercase; letter-spacing:.6px; padding:0 2px;", "Members" }
                    for m in members.read().iter() {
                        MemberRow { member: m.clone(), view }
                    }
                }
            }
        }
    }
}

#[component]
fn MemberRow(member: Member, view: Signal<MemberView>) -> Element {
    let address = member.address.clone();
    let copy_address = move |_| {
        if let Some(addr) = &address {
            let js = format!("navigator.clipboard.writeText('{addr}').catch(()=>{{}})");
            let _ = dioxus::document::eval(&js);
        }
    };

    rsx! {
        div {
            style: "background:#fff; border-radius:14px; padding:14px; box-shadow:0 1px 4px rgba(0,0,0,.07); display:flex; flex-direction:column; gap:8px; cursor:pointer;",
            onclick: {
                let m = member.clone();
                move |_| { view.set(MemberView::Detail(m.clone())); }
            },

            div {
                style: "display:flex; align-items:center; gap:12px;",
                div {
                    style: "width:40px; height:40px; border-radius:50%; background:#eef2ff; display:flex; align-items:center; justify-content:center; font-size:20px; flex-shrink:0;",
                    "{member.avatar()}"
                }
                div { style: "flex:1;",
                    div {
                        style: "display:flex; align-items:center; gap:6px; margin-bottom:2px;",
                        span { style: "font-size:14px; font-weight:700; color:#1f2937;", "{member.full_name}" }
                        if member.role == crate::models::member::MemberRole::Owner {
                            span {
                                style: "font-size:9px; font-weight:700; background:#fef3c7; color:#d97706; border-radius:4px; padding:1px 5px;",
                                "OWNER"
                            }
                        }
                    }
                    if let Some(phone) = &member.phone {
                        div { style: "font-size:12px; color:#6b7280;", "📞 {phone}" }
                    }
                }
                if member.address.is_some() {
                    button {
                        style: "background:#f3f4f6; color:#6b7280; border:none; border-radius:8px; padding:6px 10px; font-size:11px; font-weight:600; cursor:pointer; flex-shrink:0;",
                        onclick: copy_address,
                        "Copy Address"
                    }
                }
                span { style: "color:#d1d5db; font-size:18px; flex-shrink:0;", "›" }
            }
        }
    }
}
