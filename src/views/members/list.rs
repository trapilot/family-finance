use dioxus::prelude::*;

use crate::app::AppState;
use crate::models::{member::MemberRole, Family, Member};
use crate::repository::{FamilyRepo, MemberRepo};
use super::MemberView;

#[component]
pub fn MemberList(view: Signal<MemberView>) -> Element {
    let state = use_context::<AppState>();
    let families_with_members = use_signal(|| vec![]);
    let unassigned_members = use_signal(|| vec![]);

    let mut families_effect = families_with_members.clone();
    let mut unassigned_effect = unassigned_members.clone();
    let state_effect = state.clone();

    use_effect(move || {
        state_effect.with_conn(|conn| {
            if let Ok(families) = FamilyRepo::list_with_members(conn) {
                families_effect.set(families);
            }
            if let Ok(unassigned) = MemberRepo::list_unassigned(conn) {
                unassigned_effect.set(unassigned);
            }
        });
    });

    rsx! {
        div {
            style: "padding:16px; display:flex; flex-direction:column; gap:14px;",

            // ── Header ──
            div {
                style: "display:flex; justify-content:space-between; align-items:center; gap:8px; flex-wrap: wrap;",
                h1 { style: "margin:0; font-size:20px; font-weight:700; color:#1f2937;", "Families & Members" }
                div { style: "display:flex; gap:8px;",
                    button {
                        style: "background:#6366f1; color:#fff; border:none; border-radius:20px; padding:8px 16px; font-size:14px; font-weight:600; cursor:pointer;",
                        onclick: move |_| { view.set(MemberView::AddFamily); },
                        "+ Add Family"
                    }
                    button {
                        style: "background:#10b981; color:#fff; border:none; border-radius:20px; padding:8px 16px; font-size:14px; font-weight:600; cursor:pointer;",
                        onclick: move |_| { view.set(MemberView::Add); },
                        "+ Add Member"
                    }
                }
            }

            if families_with_members.read().is_empty() && unassigned_members.read().is_empty() {
                EmptyState {}
            }

            // Families
            for (family, members) in families_with_members.read().iter() {
                FamilyCard { family: family.clone(), members: members.clone(), view }
            }

            // Unassigned members
            if !unassigned_members.read().is_empty() {
                GroupLabel { label: "Unassigned Members" }
                for m in unassigned_members.read().iter() {
                    ContactRow { member: m.clone(), view }
                }
            }
        }
    }
}

// ─── FamilyCard ───────────────────────────────────────────────────────────────

#[component]
fn FamilyCard(family: Family, members: Vec<Member>, view: Signal<MemberView>) -> Element {
    let common_address_preview = family.common_address.as_deref()
        .map(|a| if a.len() > 50 { format!("{}…", &a[..50]) } else { a.to_string() })
        .unwrap_or_else(|| "—".into());

    rsx! {
        div {
            style: "background:#fff; border-radius:14px; padding:14px; box-shadow:0 1px 4px rgba(0,0,0,.07); display:flex; flex-direction:column; gap:10px; cursor:pointer;",
            onclick: {
                let family = family.clone();
                move |_| { view.set(MemberView::FamilyDetail(family.clone())); }
            },

            div {
                style: "display:flex; align-items:center; justify-content:space-between;",
                div {
                    style: "display:flex; align-items:center; gap:8px;",
                    div {
                        style: "width:40px; height:40px; border-radius:50%; background:#dbeafe; display:flex; align-items:center; justify-content:center; font-size:20px;",
                        "🏠"
                    }
                    div {
                        span { style: "font-size:16px; font-weight:700; color:#1f2937;", "{family.name}" }
                        if family.common_address.is_some() {
                            div {
                                style: "font-size:12px; color:#6b7280;", "📍 {common_address_preview}"
                            }
                        }
                    }
                }
                span { style: "color:#d1d5db; font-size:18px;", "›" }
            }

            if !members.is_empty() {
                div {
                    style: "display:flex; flex-direction:column; gap:8px; padding-left:48px;",
                    for m in members.iter() {
                        MiniContactRow { member: m.clone(), view }
                    }
                }
            }
        }
    }
}

// ─── MiniContactRow ───────────────────────────────────────────────────────────────

#[component]
fn MiniContactRow(member: Member, view: Signal<MemberView>) -> Element {
    rsx! {
        div {
            style: "display:flex; align-items:center; gap:8px; cursor:pointer; padding:4px 0;",
            onclick: {
                let m = member.clone();
                move |_| { view.set(MemberView::Detail(m.clone())); }
            },
            div {
                style: "width:32px; height:32px; border-radius:50%; background:#f3f4f6; display:flex; align-items:center; justify-content:center; font-size:18px; flex-shrink:0;",
                "{member.avatar()}"
            }
            div { style: "flex:1;",
                span { style: "font-size:14px; font-weight:500; color:#374151;", "{member.full_name}" }
            }
            span { style: "color:#d1d5db; font-size:14px; flex-shrink:0;", "›" }
        }
    }
}

// ─── ContactRow ───────────────────────────────────────────────────────────────

#[component]
fn ContactRow(member: Member, view: Signal<MemberView>) -> Element {
    let phone_preview   = member.phone.as_deref().unwrap_or("—");
    let address_preview = member.address.as_deref()
        .map(|a| if a.len() > 35 { format!("{}…", &a[..35]) } else { a.to_string() })
        .unwrap_or_else(|| "—".into());
    let is_owner = member.role == MemberRole::Owner;

    rsx! {
        div {
            style: "background:#fff; border-radius:14px; padding:14px; box-shadow:0 1px 4px rgba(0,0,0,.07); display:flex; align-items:center; gap:12px; cursor:pointer;",
            onclick: {
                let m = member.clone();
                move |_| { view.set(MemberView::Detail(m.clone())); }
            },

            // Avatar
            div {
                style: "width:48px; height:48px; border-radius:50%; background:#eef2ff; display:flex; align-items:center; justify-content:center; font-size:26px; flex-shrink:0;",
                "{member.avatar()}"
            }

            div { style: "flex:1; min-width:0;",
                div {
                    style: "display:flex; align-items:center; gap:6px; margin-bottom:3px;",
                    span { style: "font-size:15px; font-weight:700; color:#1f2937;", "{member.full_name}" }
                    if is_owner {
                        span {
                            style: "font-size:10px; font-weight:700; background:#fef3c7; color:#d97706; border-radius:4px; padding:1px 6px;",
                            "OWNER"
                        }
                    }
                }
                div { style: "font-size:12px; color:#6b7280;", "📞 {phone_preview}" }
                div {
                    style: "font-size:12px; color:#9ca3af; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;",
                    "📍 {address_preview}"
                }
            }

            span { style: "color:#d1d5db; font-size:18px; flex-shrink:0;", "›" }
        }
    }
}

#[component]
fn GroupLabel(label: &'static str) -> Element {
    rsx! {
        div {
            style: "font-size:11px; font-weight:700; color:#9ca3af; text-transform:uppercase; letter-spacing:.6px; padding:0 2px;",
            "{label}"
        }
    }
}

#[component]
fn EmptyState() -> Element {
    rsx! {
        div {
            style: "text-align:center; padding:56px 0; color:#9ca3af;",
            p { style: "font-size:48px; margin:0;", "👨‍👩‍👧‍👦" }
            p { style: "font-size:15px; font-weight:600; margin:12px 0 4px; color:#6b7280;", "No families yet" }
            p { style: "font-size:13px; margin:0;", "Add a family and members to organize contacts!" }
        }
    }
}
