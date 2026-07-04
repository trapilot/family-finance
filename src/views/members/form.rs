use dioxus::prelude::*;

use crate::app::AppState;
use crate::models::{Family, Member, NewMember};
use crate::models::member::{Gender, MemberRole};
use crate::repository::{FamilyRepo, MemberRepo};
use crate::views::date_str_to_ts;
use super::MemberView;

const AVATAR_PRESETS: &[&str] = &[
    "👨","👩","👦","👧","🧑","👴","👵","🧒",
    "🧔","👱","🧑‍💼","👩‍💼","🧑‍🎓","👩‍🎓",
    "🧑‍🍳","👩‍🍳","🧑‍🔬","🧑‍🌾",
];

#[component]
pub fn MemberForm(view: Signal<MemberView>, editing: Option<Member>) -> Element {
    let state = use_context::<AppState>();
    let families = use_signal(|| vec![]);

    let mut full_name      = use_signal(|| editing.as_ref().map(|m| m.full_name.clone()).unwrap_or_default());
    let mut family_id      = use_signal(|| editing.as_ref().and_then(|m| m.family_id.clone()));
    let mut role           = use_signal(|| editing.as_ref().map(|m| m.role.clone()).unwrap_or(MemberRole::Member));
    let mut gender         = use_signal(|| editing.as_ref().and_then(|m| m.gender.clone()));
    let mut avatar_emoji   = use_signal(|| editing.as_ref().and_then(|m| m.avatar_emoji.clone()).unwrap_or_default());
    let mut phone          = use_signal(|| editing.as_ref().and_then(|m| m.phone.clone()).unwrap_or_default());
    let mut address        = use_signal(|| editing.as_ref().and_then(|m| m.address.clone()).unwrap_or_default());
    let mut birth_date     = use_signal(|| date_from_ts(editing.as_ref().and_then(|m| m.birth_date)));
    let mut id_number      = use_signal(|| editing.as_ref().and_then(|m| m.id_number.clone()).unwrap_or_default());
    let mut id_issue_date  = use_signal(|| date_from_ts(editing.as_ref().and_then(|m| m.id_issue_date)));
    let mut id_issue_place = use_signal(|| editing.as_ref().and_then(|m| m.id_issue_place.clone()).unwrap_or_default());
    let mut note           = use_signal(|| editing.as_ref().and_then(|m| m.note.clone()).unwrap_or_default());
    let mut show_id        = use_signal(|| false);
    let error_msg          = use_signal(|| String::new());

    let title = if editing.is_some() { "Edit Member" } else { "New Member" };

    // Load families on mount
    let mut families_effect = families.clone();
    let state_effect = state.clone();
    use_effect(move || {
        state_effect.with_conn(|conn| {
            if let Ok(list) = FamilyRepo::list(conn) {
                families_effect.set(list);
            }
        });
    });

    let submit = {
        let state = state.clone();
        let editing = editing.clone();
        let mut error_msg = error_msg.clone();
        let family_id_clone = family_id.clone();

        move |_: MouseEvent| {
            let name = full_name.read().trim().to_string();
            if name.is_empty() { error_msg.set("Full name is required".into()); return; }

            let opt = |s: &str| if s.trim().is_empty() { None } else { Some(s.trim().to_string()) };

            let input = NewMember {
                family_id:      family_id_clone.read().clone(),
                full_name:      name,
                birth_date:     date_str_to_ts(&birth_date.read()),
                gender:         gender.read().clone(),
                phone:          opt(&phone.read()),
                id_number:      opt(&id_number.read()),
                id_issue_date:  date_str_to_ts(&id_issue_date.read()),
                id_issue_place: opt(&id_issue_place.read()),
                address:        opt(&address.read()),
                role:           role.read().clone(),
                avatar_emoji:   opt(&avatar_emoji.read()),
                note:           opt(&note.read()),
            };

            let result = state.with_conn(|conn| {
                if let Some(ref old) = editing {
                    let mut updated = old.clone();
                    updated.family_id      = input.family_id.clone();
                    updated.full_name      = input.full_name.clone();
                    updated.birth_date     = input.birth_date;
                    updated.gender         = input.gender.clone();
                    updated.phone          = input.phone.clone();
                    updated.id_number      = input.id_number.clone();
                    updated.id_issue_date  = input.id_issue_date;
                    updated.id_issue_place = input.id_issue_place.clone();
                    updated.address        = input.address.clone();
                    updated.role           = input.role.clone();
                    updated.avatar_emoji   = input.avatar_emoji.clone();
                    updated.note           = input.note.clone();
                    MemberRepo::update(conn, &updated)
                } else {
                    MemberRepo::create(conn, &input).map(|_| ())
                }
            });

            match result {
                Ok(_)  => { view.set(MemberView::List); }
                Err(e) => { error_msg.set(e.to_string()); }
            }
        }
    };

    // "{if avatar_emoji.read().is_empty() { \"🧑\".to_string() } else { avatar_emoji.read().clone() }}"
    
    let avatar_icon = if avatar_emoji.read().is_empty() { 
        "🧑" 
    } else { 
        &avatar_emoji.read().clone().to_string()
    };

    rsx! {
        div {
            style: "padding:16px; display:flex; flex-direction:column; gap:14px; padding-bottom:40px;",

            // ── Header ──
            div {
                style: "display:flex; align-items:center; gap:12px;",
                button {
                    style: "border:none; background:transparent; font-size:22px; cursor:pointer; padding:0;",
                    onclick: move |_| { view.set(MemberView::List); },
                    "←"
                }
                h1 { style: "margin:0; font-size:18px; font-weight:700; color:#1f2937;", "{title}" }
            }

            // ── Avatar picker ──
            Label { text: "Avatar" }
            div {
                style: "display:flex; align-items:center; gap:12px; margin-bottom:4px;",
                div {
                    style: "width:58px; height:58px; border-radius:50%; background:#eef2ff; display:flex; align-items:center; justify-content:center; font-size:32px; border:2px solid #c7d2fe;",
                    {avatar_icon}
                }
            }
            div {
                style: "display:flex; flex-wrap:wrap; gap:8px;",
                for emoji in AVATAR_PRESETS.iter() {
                    {
                        let e = emoji.to_string();
                        let is_sel = *avatar_emoji.read() == *emoji;
                        rsx! {
                            button {
                                style: if is_sel {
                                    "width:42px; height:42px; border:2px solid #6366f1; border-radius:10px; background:#eef2ff; font-size:22px; cursor:pointer;"
                                } else {
                                    "width:42px; height:42px; border:1px solid #e5e7eb; border-radius:10px; background:#fff; font-size:22px; cursor:pointer;"
                                },
                                onclick: move |_| { avatar_emoji.set(e.clone()); },
                                "{emoji}"
                            }
                        }
                    }
                }
            }

            // ── Full name ──
            Label { text: "Full Name *" }
            TextInput { value: full_name.read().clone(), placeholder: "", on_input: move |v| { full_name.set(v); } }

            // ── Family ──
            Label { text: "Family" }
            div {
                style: "display:flex; flex-direction:column; gap:6px;",
                select {
                    style: "border:1px solid #e5e7eb; border-radius:8px; padding:10px 12px; font-size:15px; outline:none; background:#fff; width:100%; box-sizing:border-box;",
                    oninput: move |e| {
                        let val = e.value();
                        if val.is_empty() {
                            family_id.set(None);
                        } else {
                            family_id.set(Some(val));
                        }
                    },
                    option {
                        value: "",
                        selected: family_id.read().is_none(),
                        "Unassigned"
                    }
                    for family in families.read().iter() {
                        option {
                            value: "{family.id}",
                            selected: family_id.read().as_deref() == Some(&family.id),
                            "{family.name}"
                        }
                    }
                }
            }

            // ── Role ──
            Label { text: "Role" }
            div {
                style: "display:flex; gap:8px;",
                for (r, label, icon) in [(MemberRole::Owner,"Owner","👑"),(MemberRole::Member,"Member","👤")] {
                    {
                        let rc = r.clone();
                        let is_sel = *role.read() == r;
                        rsx! {
                            button {
                                style: if is_sel {
                                    "flex:1; padding:10px; border:2px solid #6366f1; border-radius:10px; background:#eef2ff; color:#6366f1; font-size:13px; font-weight:700; cursor:pointer;"
                                } else {
                                    "flex:1; padding:10px; border:1px solid #e5e7eb; border-radius:10px; background:#fff; color:#6b7280; font-size:13px; cursor:pointer;"
                                },
                                onclick: move |_| { role.set(rc.clone()); },
                                "{icon} {label}"
                            }
                        }
                    }
                }
            }

            // ── Gender ──
            Label { text: "Gender" }
            div {
                style: "display:flex; gap:8px;",
                for (g, label) in [(Some(Gender::Male),"Male"),(Some(Gender::Female),"Female"),(None,"Other")] {
                    {
                        let gc = g.clone();
                        let is_sel = *gender.read() == g;
                        rsx! {
                            button {
                                style: if is_sel {
                                    "flex:1; padding:9px; border:none; border-radius:8px; background:#6366f1; color:#fff; font-size:13px; font-weight:600; cursor:pointer;"
                                } else {
                                    "flex:1; padding:9px; border:1px solid #e5e7eb; border-radius:8px; background:#fff; color:#6b7280; font-size:13px; cursor:pointer;"
                                },
                                onclick: move |_| { gender.set(gc.clone()); },
                                "{label}"
                            }
                        }
                    }
                }
            }

            // ── Phone ──
            Label { text: "Phone" }
            TextInput { value: phone.read().clone(), placeholder: "", on_input: move |v| { phone.set(v); } }

            // ── Address (the main use-case: copy for Grab/Uber) ──
            Label { text: "Address" }
            div {
                style: "position:relative;",
                textarea {
                    style: "width:100%; border:1px solid #e5e7eb; border-radius:8px; padding:10px 12px; font-size:14px; outline:none; background:#fff; resize:none; height:72px; box-sizing:border-box; font-family:inherit;",
                    placeholder: "",
                    value: "{address.read()}",
                    oninput: move |e| { address.set(e.value()); },
                }
            }
            div { style: "font-size:11px; color:#9ca3af; margin-top:-8px;",
                "💡 Đây là địa chỉ bạn sẽ copy khi đặt xe tới nhà người này."
            }

            // ── Date of birth ──
            Label { text: "Date of Birth" }
            input {
                style: "border:1px solid #e5e7eb; border-radius:8px; padding:10px 12px; font-size:14px; outline:none; background:#fff;",
                r#type: "date",
                value: "{birth_date.read()}",
                oninput: move |e| { birth_date.set(e.value()); },
            }

            // ── Identity (collapsible) ──
            button {
                style: "text-align:left; border:none; background:transparent; padding:4px 0; font-size:13px; font-weight:600; color:#6366f1; cursor:pointer;",
                onclick: move |_| show_id.toggle(),
                if *show_id.read() { "▼ Identity Document (CCCD/CMND)" }
                else               { "▶ Identity Document (CCCD/CMND)" }
            }

            if *show_id.read() {
                div {
                    style: "background:#f9fafb; border-radius:10px; padding:14px; display:flex; flex-direction:column; gap:12px;",
                    Label { text: "ID Number" }
                    TextInput { value: id_number.read().clone(), placeholder: "", on_input: move |v| { id_number.set(v); } }
                    Label { text: "Issue Date" }
                    input {
                        style: "border:1px solid #e5e7eb; border-radius:8px; padding:10px 12px; font-size:14px; outline:none; background:#fff;",
                        r#type: "date",
                        value: "{id_issue_date.read()}",
                        oninput: move |e| { id_issue_date.set(e.value()); },
                    }
                    Label { text: "Issue Place" }
                    TextInput { value: id_issue_place.read().clone(), placeholder: "", on_input: move |v| { id_issue_place.set(v); } }
                }
            }

            // ── Note ──
            Label { text: "Note (optional)" }
            TextInput { value: note.read().clone(), placeholder: "", on_input: move |v| { note.set(v); } }

            // ── Error ──
            if !error_msg.read().is_empty() {
                div {
                    style: "background:#fef2f2; border-radius:8px; padding:10px 12px; font-size:13px; color:#ef4444;",
                    "{error_msg.read()}"
                }
            }

            button {
                style: "padding:14px; background:#6366f1; color:#fff; border:none; border-radius:12px; font-size:16px; font-weight:700; cursor:pointer;",
                onclick: submit,
                if editing.is_some() { "Update Member" } else { "Add Member" }
            }
        }
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn date_from_ts(ts: Option<i64>) -> String {
    ts.and_then(|t| {
        use chrono::{Local, TimeZone};
        Local.timestamp_opt(t, 0).single()
            .map(|d| d.format("%Y-%m-%d").to_string())
    }).unwrap_or_default()
}

#[component]
fn Label(text: &'static str) -> Element {
    rsx! {
        label {
            style: "font-size:12px; font-weight:600; color:#374151; text-transform:uppercase; letter-spacing:.5px;",
            "{text}"
        }
    }
}

#[component]
fn TextInput(value: String, placeholder: &'static str, on_input: EventHandler<String>) -> Element {
    rsx! {
        input {
            style: "border:1px solid #e5e7eb; border-radius:8px; padding:10px 12px; font-size:15px; outline:none; background:#fff; width:100%; box-sizing:border-box;",
            r#type: "text",
            placeholder,
            value: "{value}",
            oninput: move |e| { on_input.call(e.value()); },
        }
    }
}
