use dioxus::prelude::*;

use crate::app::AppState;
use crate::models::{member::MemberRole, Member};
use crate::repository::MemberRepo;
use crate::views::ts_to_date_str;
use super::MemberView;

#[component]
pub fn MemberDetail(view: Signal<MemberView>, member: Member) -> Element {
    let state          = use_context::<AppState>();
    let mut confirm_delete = use_signal(|| false);
    let mut copied_field   = use_signal(|| None::<&'static str>);

    // Helper: copy text to clipboard via JS eval in WebView
    let copy = move |text: String, field: &'static str| {
        let js = format!("navigator.clipboard.writeText('{text}').catch(()=>{{}})");
        let _ = dioxus::document::eval(&js);
        copied_field.set(Some(field));
        // Clear "Copied!" badge after 1.5s
        spawn(async move {
            // tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
            copied_field.set(None);
        });
    };
    
    let member_editable = member.clone();

    rsx! {
        div {
            style: "display:flex; flex-direction:column; height:100%;",

            // ── Top bar ──
            div {
                style: "display:flex; align-items:center; gap:10px; padding:14px 16px; background:#fff; border-bottom:1px solid #f3f4f6;",
                button {
                    style: "border:none; background:transparent; font-size:22px; cursor:pointer; padding:0; flex-shrink:0;",
                    onclick: move |_| { view.set(MemberView::List); },
                    "←"
                }
                div { style: "flex:1;" }
                button {
                    style: "border:none; background:#f3f4f6; border-radius:8px; padding:7px 14px; font-size:13px; font-weight:600; cursor:pointer; color:#374151;",
                    onclick: {
                        move |_| { view.set(MemberView::Edit(member_editable.clone())); }
                    },
                    "Edit"
                }
                button {
                    style: "border:none; background:#fef2f2; border-radius:8px; padding:7px 14px; font-size:13px; font-weight:600; cursor:pointer; color:#ef4444;",
                    onclick: move |_| { confirm_delete.set(true); },
                    "Delete"
                }
            }

            div {
                style: "flex:1; overflow-y:auto;",

                // ── Hero card ──
                div {
                    style: "background:linear-gradient(135deg,#6366f1,#8b5cf6); padding:28px 20px 24px; display:flex; flex-direction:column; align-items:center; gap:10px;",
                    div {
                        style: "width:80px; height:80px; border-radius:50%; background:rgba(255,255,255,.2); display:flex; align-items:center; justify-content:center; font-size:44px; border:3px solid rgba(255,255,255,.4);",
                        "{member.avatar()}"
                    }
                    div { style: "font-size:22px; font-weight:800; color:#fff; text-align:center;", "{member.full_name}" }
                    div {
                        style: "display:flex; gap:8px; align-items:center;",
                        if member.role == MemberRole::Owner {
                            span {
                                style: "font-size:11px; font-weight:700; background:#fef3c7; color:#d97706; border-radius:6px; padding:3px 8px;",
                                "👑 OWNER"
                            }
                        } else {
                            span {
                                style: "font-size:11px; font-weight:700; background:rgba(255,255,255,.2); color:#fff; border-radius:6px; padding:3px 8px;",
                                "👤 MEMBER"
                            }
                        }
                        if let Some(ref gender) = member.gender {
                            span {
                                style: "font-size:11px; background:rgba(255,255,255,.15); color:#fff; border-radius:6px; padding:3px 8px;",
                                "{gender}"
                            }
                        }
                    }
                }

                // ── Contact fields ──
                div {
                    style: "padding:16px; display:flex; flex-direction:column; gap:10px;",

                    // Phone
                    if let Some(ref phone) = member.phone {
                        {
                            let phone = phone.clone();
                            let mut copy = copy.clone();
                            rsx! {
                                CopyField {
                                    icon:  "📞",
                                    label: "Phone",
                                    value: phone.clone(),
                                    copied: *copied_field.read() == Some("phone"),
                                    on_copy: move |_| { copy(phone.clone(), "phone"); }
                                }
                            }
                        }
                    }

                    // Address
                    if let Some(ref address) = member.address {
                        {
                            let address = address.clone();
                            let mut copy = copy.clone();
                            rsx! {
                                CopyField {
                                    icon:  "📍",
                                    label: "Address",
                                    value: address.clone(),
                                    copied: *copied_field.read() == Some("address"),
                                    on_copy: move |_| { copy(address.clone(), "address"); }
                                }
                            }
                        }
                    }

                    // ID Number
                    if let Some(ref id_number) = member.id_number {
                        {
                            let id_number = id_number.clone();
                            let mut copy = copy.clone();
                            rsx! {
                                CopyField {
                                    icon:  "🪪",
                                    label: "ID Number (CCCD/CMND)",
                                    value: id_number.clone(),
                                    copied: *copied_field.read() == Some("id_number"),
                                    on_copy: move |_| { copy(id_number.clone(), "id_number"); }
                                }
                            }
                        }
                    }

                    // ID Issue info
                    if member.id_issue_date.is_some() || member.id_issue_place.is_some() {
                        div {
                            style: "background:#fff; border-radius:12px; padding:14px; box-shadow:0 1px 3px rgba(0,0,0,.06);",
                            div { style: "font-size:11px; font-weight:700; color:#9ca3af; text-transform:uppercase; margin-bottom:8px;", "ID Issue" }
                            if let Some(ts) = member.id_issue_date {
                                InfoRow { label: "Issue Date",  value: ts_to_date_str(ts) }
                            }
                            if let Some(ref place) = member.id_issue_place {
                                InfoRow { label: "Issue Place", value: place.clone() }
                            }
                        }
                    }

                    // Personal info
                    div {
                        style: "background:#fff; border-radius:12px; padding:14px; box-shadow:0 1px 3px rgba(0,0,0,.06);",
                        div { style: "font-size:11px; font-weight:700; color:#9ca3af; text-transform:uppercase; margin-bottom:8px;", "Personal Info" }
                        if let Some(bd) = member.birth_date {
                            InfoRow { label: "Date of Birth", value: ts_to_date_str(bd) }
                        }
                        if let Some(ref gender) = member.gender {
                            InfoRow { label: "Gender", value: gender.to_string() }
                        }
                        InfoRow { label: "Role", value: member.role.to_string() }
                    }

                    // Note
                    if let Some(ref note) = member.note {
                        div {
                            style: "background:#fff; border-radius:12px; padding:14px; box-shadow:0 1px 3px rgba(0,0,0,.06);",
                            div { style: "font-size:11px; font-weight:700; color:#9ca3af; text-transform:uppercase; margin-bottom:6px;", "Note" }
                            div { style: "font-size:14px; color:#374151; line-height:1.5;", "{note}" }
                        }
                    }
                }
            }

            // ── Delete confirm modal ──
            if *confirm_delete.read() {
                div {
                    style: "position:fixed; inset:0; background:rgba(0,0,0,.45); display:flex; align-items:center; justify-content:center; z-index:100;",
                    div {
                        style: "background:#fff; border-radius:16px; padding:24px; margin:20px; max-width:320px; width:100%;",
                        h3 { style: "margin:0 0 8px; font-size:17px; font-weight:700;", "Remove member?" }
                        p  { style: "margin:0 0 20px; font-size:14px; color:#6b7280; line-height:1.5;",
                            ""{member.full_name}" will be permanently removed from your family directory."
                        }
                        div { style: "display:flex; gap:10px;",
                            button {
                                style: "flex:1; padding:11px; border:1px solid #e5e7eb; border-radius:8px; background:#fff; font-size:14px; cursor:pointer;",
                                onclick: move |_| { confirm_delete.set(false); },
                                "Cancel"
                            }
                            button {
                                style: "flex:1; padding:11px; border:none; border-radius:8px; background:#ef4444; color:#fff; font-size:14px; font-weight:700; cursor:pointer;",
                                onclick: {
                                    let state = state.clone();
                                    let mid   = member.id.clone();
                                    move |_| {
                                        state.with_conn(|conn| { let _ = MemberRepo::delete(conn, &mid); });
                                        view.set(MemberView::List);
                                    }
                                },
                                "Remove"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ─── CopyField ────────────────────────────────────────────────────────────────
// Tap the copy button → text goes to clipboard, badge flashes "Copied!"

#[component]
fn CopyField(
    icon:    &'static str,
    label:   &'static str,
    value:   String,
    copied:  bool,
    on_copy: EventHandler<MouseEvent>,
) -> Element {
    let button_style = if copied { 
        "background:#10b981; color:#fff;" 
    } else { 
        "background:#f3f4f6; color:#374151;" 
    };

    rsx! {
        div {
            style: "background:#fff; border-radius:12px; padding:14px; box-shadow:0 1px 3px rgba(0,0,0,.06); display:flex; align-items:flex-start; gap:12px;",

            span { style: "font-size:22px; flex-shrink:0; margin-top:1px;", "{icon}" }

            div { style: "flex:1; min-width:0;",
                div { style: "font-size:11px; font-weight:700; color:#9ca3af; text-transform:uppercase; margin-bottom:3px;", "{label}" }
                div { style: "font-size:15px; font-weight:500; color:#1f2937; word-break:break-word;", "{value}" }
            }

            button {
                style: format!(
                    "flex-shrink:0; border:none; border-radius:8px; padding:7px 12px; font-size:12px; font-weight:600; cursor:pointer; transition:all .15s; {}", 
                    button_style
                ),
                onclick: move |e| { on_copy.call(e); },
                if copied { "✓ Copied" } else { "Copy" }
            }
        }
    }
}

// ─── InfoRow ─────────────────────────────────────────────────────────────────

#[component]
fn InfoRow(label: &'static str, value: String) -> Element {
    rsx! {
        div {
            style: "display:flex; justify-content:space-between; align-items:baseline; padding:5px 0; border-bottom:1px solid #f9fafb;",
            span { style: "font-size:13px; color:#9ca3af; flex-shrink:0; margin-right:12px;", "{label}" }
            span { style: "font-size:13px; font-weight:600; color:#374151; text-align:right; word-break:break-all;", "{value}" }
        }
    }
}
