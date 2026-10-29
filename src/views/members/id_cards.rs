use base64::Engine;
use dioxus::prelude::*;
use std::collections::HashMap;

use crate::app::AppState;
use crate::i18n::use_i18n;
use crate::models::{id_card::jpeg_data_url, IdSide, Member};
use crate::repository::{FamilyRepo, IdCardRepo, MemberRepo};
use crate::services::id_card_pdf::{self, CardPair};
use super::MemberView;

const B64: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD;

// ─── Photo picker (member form) ───────────────────────────────────────────────

/// Dioxus desktop/mobile swaps clicks on file inputs for its own native dialog, which
/// is empty on iOS. Our inputs (`data-id-photo`) stop the click in the capture phase so
/// the WebView's own camera / photo-library picker opens; the chosen photo is shrunk to
/// ≤1600px JPEG here and sent back as `{side, data}` (data = base64, "" on failure).
const PICK_PHOTO_JS: &str = r#"
    const old = window.__ffIdPhoto;
    if (old) {
        document.removeEventListener('click', old.click, true);
        document.removeEventListener('change', old.change, true);
    }
    const ours = t => t instanceof HTMLInputElement && t.dataset.idPhoto;
    const click = e => { if (ours(e.target)) e.stopPropagation(); };
    const change = e => {
        const input = e.target;
        if (!ours(input)) return;
        e.stopPropagation();
        const side = input.dataset.idPhoto;
        const file = input.files && input.files[0];
        input.value = '';
        if (!file) return;
        const url = URL.createObjectURL(file);
        const img = new Image();
        img.onload = () => {
            const s = Math.min(1, 1600 / Math.max(img.naturalWidth, img.naturalHeight));
            const c = document.createElement('canvas');
            c.width = Math.round(img.naturalWidth * s);
            c.height = Math.round(img.naturalHeight * s);
            const ctx = c.getContext('2d');
            ctx.fillStyle = '#fff';
            ctx.fillRect(0, 0, c.width, c.height);
            ctx.drawImage(img, 0, 0, c.width, c.height);
            URL.revokeObjectURL(url);
            dioxus.send({ side, data: c.toDataURL('image/jpeg', 0.85).split(',')[1] });
        };
        img.onerror = () => { URL.revokeObjectURL(url); dioxus.send({ side, data: '' }); };
        img.src = url;
    };
    document.addEventListener('click', click, true);
    document.addEventListener('change', change, true);
    window.__ffIdPhoto = { click, change };
"#;

/// What `PICK_PHOTO_JS` sends for each chosen photo
#[derive(serde::Deserialize)]
struct PickedPhoto {
    side: String,
    data: String,
}

/// Front + back photo slots. Edits stay in the signals until the form is saved.
#[component]
pub fn IdPhotoFields(front: Signal<Option<Vec<u8>>>, back: Signal<Option<Vec<u8>>>) -> Element {
    let t = use_i18n();
    let mut error = use_signal(|| false);

    use_hook(move || {
        spawn(async move {
            let mut eval = document::eval(PICK_PHOTO_JS);
            while let Ok(msg) = eval.recv::<PickedPhoto>().await {
                let mut target = if msg.side == "back" { back } else { front };
                match B64.decode(&msg.data) {
                    Ok(jpeg) if !jpeg.is_empty() => { target.set(Some(jpeg)); error.set(false); }
                    _                            => error.set(true),
                }
            }
        });
    });

    rsx! {
        div {
            style: "display:flex; gap:10px;",
            PhotoSlot { side: IdSide::Front, photo: front }
            PhotoSlot { side: IdSide::Back,  photo: back }
        }
        if error() {
            div { style: "font-size:12px; color:#ef4444;", {t.t("❌ Could not read that photo")} }
        }
    }
}

#[component]
fn PhotoSlot(side: IdSide, photo: Signal<Option<Vec<u8>>>) -> Element {
    let t = use_i18n();
    let label = side_label(side);
    let src = photo.read().as_deref().map(jpeg_data_url);

    rsx! {
        div {
            style: "flex:1; min-width:0; display:flex; flex-direction:column; gap:6px;",
            div { style: "font-size:12px; font-weight:600; color:#6b7280;", {t.t(label)} }
            label {
                style: "aspect-ratio:1.585; border:1px dashed #c7d2fe; border-radius:8px; background:#fff; display:flex; align-items:center; justify-content:center; overflow:hidden; cursor:pointer; font-size:13px; color:#6366f1; text-align:center;",
                if let Some(src) = src {
                    img { src, style: "width:100%; height:100%; object-fit:contain;" }
                } else {
                    {t.t("📷 Add photo")}
                }
                input {
                    r#type: "file",
                    accept: "image/*",
                    style: "display:none;",
                    "data-id-photo": "{side}",
                }
            }
            if photo.read().is_some() {
                button {
                    style: "border:none; background:transparent; color:#ef4444; font-size:12px; cursor:pointer; padding:0;",
                    onclick: move |_| photo.set(None),
                    {t.t("Remove")}
                }
            }
        }
    }
}

/// Write the form's photos for a member: present → saved, cleared → deleted
pub fn save_photos(
    conn: &rusqlite::Connection,
    member_id: &str,
    front: Option<&[u8]>,
    back: Option<&[u8]>,
) -> crate::error::Result<()> {
    for (side, jpeg) in [(IdSide::Front, front), (IdSide::Back, back)] {
        match jpeg {
            Some(jpeg) => IdCardRepo::set(conn, member_id, side, jpeg)?,
            None       => IdCardRepo::remove(conn, member_id, side)?,
        }
    }
    Ok(())
}

fn side_label(side: IdSide) -> &'static str {
    match side {
        IdSide::Front => "Front side",
        IdSide::Back  => "Back side",
    }
}

// ─── Photo preview (member detail) ────────────────────────────────────────────

#[component]
pub fn IdPhotoPreview(member_id: String) -> Element {
    let t = use_i18n();
    let state = use_context::<AppState>();
    let photos: Vec<(IdSide, String)> = state.with_conn(|conn| {
        IdSide::ALL.iter()
            .filter_map(|&side| IdCardRepo::get(conn, &member_id, side).ok().flatten().map(|j| (side, jpeg_data_url(&j))))
            .collect()
    });

    if photos.is_empty() {
        return rsx! {};
    }

    rsx! {
        div {
            style: "background:#fff; border-radius:12px; padding:14px; box-shadow:0 1px 3px rgba(0,0,0,.06);",
            div { style: "font-size:11px; font-weight:700; color:#9ca3af; text-transform:uppercase; margin-bottom:8px;", {t.t("ID Card Photos")} }
            div {
                style: "display:flex; gap:10px;",
                for (side, src) in photos {
                    div {
                        style: "flex:1; min-width:0;",
                        img { src, style: "width:100%; aspect-ratio:1.585; object-fit:contain; border-radius:6px; background:#f9fafb;" }
                        div { style: "font-size:11px; color:#9ca3af; text-align:center; margin-top:4px;", {t.t(side_label(side))} }
                    }
                }
            }
        }
    }
}

// ─── ID cards → PDF ───────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
struct Candidate {
    member:      Member,
    family_name: Option<String>,
    sides:       Vec<IdSide>,
}

/// Turn the PDF into a File held by the page, so the later Share tap is a tiny,
/// instant eval and still counts as a user gesture for `navigator.share`.
const STAGE_PDF_JS: &str = r#"
    const [b64, name] = await dioxus.recv();
    const bytes = Uint8Array.from(atob(b64), c => c.charCodeAt(0));
    window.__ffIdPdf = new File([bytes], name, { type: 'application/pdf' });
    dioxus.send('ready');
"#;

const SHARE_PDF_JS: &str = r#"
    const file = window.__ffIdPdf;
    try {
        if (navigator.canShare && navigator.canShare({ files: [file] })) {
            await navigator.share({ files: [file], title: file.name });
            dioxus.send('shared');
        } else {
            const a = document.createElement('a');
            a.href = URL.createObjectURL(file);
            a.download = file.name;
            document.body.appendChild(a);
            a.click();
            a.remove();
            dioxus.send('downloaded');
        }
    } catch (e) {
        dioxus.send(e && e.name === 'AbortError' ? 'cancelled' : 'error: ' + e);
    }
"#;

#[component]
pub fn IdCardsPdf(view: Signal<MemberView>) -> Element {
    let t = use_i18n();
    let state = use_context::<AppState>();

    let candidates = use_signal({
        let state = state.clone();
        move || state.with_conn(load_candidates).unwrap_or_default()
    });
    let mut selected = use_signal(Vec::<String>::new);  // member ids, in PDF order
    let mut drag     = use_signal(|| None::<Drag>);
    let mut ready    = use_signal(|| false);
    let mut status   = use_signal(String::new);

    // Any change to the selection makes the staged PDF stale
    use_effect(move || {
        let _ = selected.read();
        ready.set(false);
    });

    let by_id: HashMap<String, Candidate> = candidates.read().iter()
        .map(|c| (c.member.id.clone(), c.clone()))
        .collect();
    let picked: Vec<Candidate> = selected.read().iter().filter_map(|id| by_id.get(id).cloned()).collect();

    // Families with at least one photographed member, for "add whole family"
    let mut families: Vec<String> = Vec::new();
    for c in candidates.read().iter().filter(|c| !c.sides.is_empty()) {
        if let Some(name) = &c.family_name {
            if !families.contains(name) { families.push(name.clone()); }
        }
    }

    let create = {
        let state = state.clone();
        move |_| {
            let ids = selected.read().clone();
            let result = state.with_conn(|conn| -> crate::error::Result<Vec<u8>> {
                let mut people = Vec::new();
                for id in &ids {
                    people.push(CardPair {
                        front: IdCardRepo::get(conn, id, IdSide::Front)?,
                        back:  IdCardRepo::get(conn, id, IdSide::Back)?,
                    });
                }
                id_card_pdf::build(&people)
            });
            match result {
                Ok(pdf) => {
                    let name = format!("CCCD_{}.pdf", chrono::Local::now().format("%Y-%m-%d"));
                    let pages = ids.len().div_ceil(id_card_pdf::ROWS_PER_PAGE);
                    spawn(async move {
                        let mut eval = document::eval(STAGE_PDF_JS);
                        let _ = eval.send((B64.encode(&pdf), name));
                        if eval.recv::<String>().await.is_ok() {
                            ready.set(true);
                            status.set(
                                t.t("PDF ready: {people} people, {pages} page(s)")
                                    .replace("{people}", &ids.len().to_string())
                                    .replace("{pages}", &pages.to_string()),
                            );
                        }
                    });
                }
                Err(e) => status.set(format!("❌ {e}")),
            }
        }
    };

    let share = move |_| {
        spawn(async move {
            let mut eval = document::eval(SHARE_PDF_JS);
            if let Ok(result) = eval.recv::<String>().await {
                if result.starts_with("error") {
                    status.set(format!("❌ {}: {}", t.t("Share failed"), result.trim_start_matches("error: ")));
                }
            }
        });
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
                h1 { style: "margin:0; font-size:18px; font-weight:700; color:#1f2937;", {t.t("ID Cards → PDF")} }
            }
            div { style: "font-size:13px; color:#6b7280; line-height:1.5;",
                {t.t("Tap people in the order you want, then drag ☰ to reorder. Each person is one row (front | back) at real card size, 5 per A4 page.")}
            }

            // ── Selected, in order ──
            if !picked.is_empty() {
                div {
                    style: "background:#fff; border-radius:14px; padding:12px; box-shadow:0 1px 4px rgba(0,0,0,.07); display:flex; flex-direction:column; gap:6px;",
                    div { style: "display:flex; justify-content:space-between; align-items:center;",
                        span { style: "font-size:11px; font-weight:700; color:#9ca3af; text-transform:uppercase;",
                            {t.t("PDF order")}
                        }
                        button {
                            style: "border:none; background:transparent; color:#ef4444; font-size:12px; cursor:pointer;",
                            onclick: move |_| selected.write().clear(),
                            {t.t("Clear")}
                        }
                    }
                    // Pointer events (not HTML5 drag-and-drop, which iOS ignores for touch).
                    // Listening on the list keeps the drag going when the finger leaves the handle.
                    div {
                        style: "display:flex; flex-direction:column; gap:{ORDER_GAP_PX}px;",
                        onpointermove: move |e| {
                            let Some(mut d) = drag() else { return };
                            let len = selected.read().len();
                            let rows = (e.client_coordinates().y - d.start_y) / ORDER_STEP_PX;
                            let target = (d.from as f64 + rows).round().clamp(0.0, (len - 1) as f64) as usize;
                            if target != d.at {
                                let mut sel = selected.write();
                                let id = sel.remove(d.at);
                                sel.insert(target, id);
                                d.at = target;
                                drag.set(Some(d));
                            }
                        },
                        onpointerup: move |_| drag.set(None),
                        onpointercancel: move |_| drag.set(None),
                        onpointerleave: move |_| drag.set(None),
                        for (i, c) in picked.iter().enumerate() {
                            OrderRow { index: i, candidate: c.clone(), selected, drag }
                        }
                    }
                }
            }

            // ── Quick add a whole family ──
            if !families.is_empty() {
                div { style: "display:flex; flex-wrap:wrap; gap:8px;",
                    for name in families {
                        button {
                            style: "border:1px solid #c7d2fe; background:#eef2ff; color:#4f46e5; border-radius:16px; padding:6px 12px; font-size:13px; cursor:pointer;",
                            onclick: {
                                let name = name.clone();
                                move |_| {
                                    let ids: Vec<String> = candidates.read().iter()
                                        .filter(|c| !c.sides.is_empty() && c.family_name.as_deref() == Some(name.as_str()))
                                        .map(|c| c.member.id.clone())
                                        .collect();
                                    let mut sel = selected.write();
                                    for id in ids {
                                        if !sel.contains(&id) { sel.push(id); }
                                    }
                                }
                            },
                            "+ 🏠 {name}"
                        }
                    }
                }
            }

            // ── Everyone ──
            div {
                style: "background:#fff; border-radius:14px; box-shadow:0 1px 4px rgba(0,0,0,.07); overflow:hidden;",
                if candidates.read().is_empty() {
                    div { style: "padding:24px; text-align:center; color:#9ca3af; font-size:13px;", {t.t("No members yet")} }
                }
                for c in candidates.read().iter().cloned() {
                    CandidateRow { candidate: c, selected }
                }
            }

            if !status.read().is_empty() {
                div { style: "font-size:13px; color:#374151; background:#f3f4f6; border-radius:8px; padding:10px 12px;", "{status}" }
            }

            if ready() {
                button {
                    style: "padding:14px; background:#10b981; color:#fff; border:none; border-radius:12px; font-size:16px; font-weight:700; cursor:pointer;",
                    onclick: share,
                    {t.t("📤 Share / Save PDF")}
                }
            } else {
                button {
                    style: if picked.is_empty() {
                        "padding:14px; background:#c7d2fe; color:#fff; border:none; border-radius:12px; font-size:16px; font-weight:700;"
                    } else {
                        "padding:14px; background:#6366f1; color:#fff; border:none; border-radius:12px; font-size:16px; font-weight:700; cursor:pointer;"
                    },
                    disabled: picked.is_empty(),
                    onclick: create,
                    {t.t("📄 Create PDF")} " ({picked.len()})"
                }
            }
        }
    }
}

#[component]
fn CandidateRow(candidate: Candidate, selected: Signal<Vec<String>>) -> Element {
    let t = use_i18n();
    let id = candidate.member.id.clone();
    let position = selected.read().iter().position(|s| *s == id);
    let has_photos = !candidate.sides.is_empty();
    let missing = if !has_photos {
        Some(t.t("No ID photos"))
    } else if candidate.sides.len() == 1 {
        Some(if candidate.sides[0] == IdSide::Front { t.t("Back side missing") } else { t.t("Front side missing") })
    } else {
        None
    };

    rsx! {
        div {
            style: if has_photos {
                "display:flex; align-items:center; gap:10px; padding:10px 12px; border-bottom:1px solid #f3f4f6; cursor:pointer;"
            } else {
                "display:flex; align-items:center; gap:10px; padding:10px 12px; border-bottom:1px solid #f3f4f6; opacity:.45;"
            },
            onclick: move |_| {
                if !has_photos { return; }
                let mut sel = selected.write();
                match sel.iter().position(|s| *s == id) {
                    Some(i) => { sel.remove(i); }
                    None    => sel.push(id.clone()),
                }
            },
            div {
                style: if position.is_some() {
                    "width:26px; height:26px; border-radius:50%; background:#6366f1; color:#fff; font-size:13px; font-weight:700; display:flex; align-items:center; justify-content:center; flex-shrink:0;"
                } else {
                    "width:26px; height:26px; border-radius:50%; border:2px solid #d1d5db; flex-shrink:0;"
                },
                if let Some(p) = position { "{p + 1}" }
            }
            span { style: "font-size:20px;", "{candidate.member.avatar()}" }
            div { style: "flex:1; min-width:0;",
                div { style: "font-size:14px; font-weight:600; color:#1f2937;", "{candidate.member.full_name}" }
                div { style: "font-size:12px; color:#9ca3af;",
                    if let Some(f) = &candidate.family_name { "🏠 {f}" }
                    if let Some(m) = missing { span { style: "color:#d97706; margin-left:6px;", "{m}" } }
                }
            }
        }
    }
}

const ORDER_ROW_PX:  f64 = 40.0;
const ORDER_GAP_PX:  f64 = 6.0;
const ORDER_STEP_PX: f64 = ORDER_ROW_PX + ORDER_GAP_PX;

/// A row being dragged: where it started, where it is now, and the pointer's start y
#[derive(Clone, Copy, PartialEq)]
struct Drag {
    from:    usize,
    at:      usize,
    start_y: f64,
}

#[component]
fn OrderRow(index: usize, candidate: Candidate, selected: Signal<Vec<String>>, drag: Signal<Option<Drag>>) -> Element {
    let dragging = drag().is_some_and(|d| d.at == index);
    rsx! {
        div {
            style: format!(
                "height:{ORDER_ROW_PX}px; display:flex; align-items:center; gap:8px; border-radius:8px; padding:0 4px; {}",
                if dragging { "background:#eef2ff; box-shadow:0 2px 8px rgba(99,102,241,.35);" } else { "background:#fff;" }
            ),
            // Drag handle: touch-action:none stops the page scrolling while dragging
            div {
                style: "width:36px; height:100%; display:flex; align-items:center; justify-content:center; color:#9ca3af; font-size:18px; cursor:grab; touch-action:none; user-select:none; -webkit-user-select:none;",
                onpointerdown: move |e| {
                    e.prevent_default();
                    drag.set(Some(Drag { from: index, at: index, start_y: e.client_coordinates().y }));
                },
                "☰"
            }
            span { style: "width:20px; font-size:13px; font-weight:700; color:#6366f1; text-align:right;", "{index + 1}." }
            span { style: "flex:1; min-width:0; font-size:14px; color:#1f2937; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;",
                "{candidate.member.full_name}"
            }
            button {
                style: "border:none; background:transparent; color:#9ca3af; font-size:16px; cursor:pointer; width:28px;",
                onclick: move |_| { selected.write().remove(index); },
                "✕"
            }
        }
    }
}

/// Everyone, family by family (same order as the Family tab), unassigned last
fn load_candidates(conn: &rusqlite::Connection) -> crate::error::Result<Vec<Candidate>> {
    let mut sides = IdCardRepo::sides_by_member(conn)?;
    let mut out = Vec::new();
    let mut push = |member: Member, family_name: Option<String>| {
        let sides = sides.remove(&member.id).unwrap_or_default();
        out.push(Candidate { member, family_name, sides });
    };
    for (family, members) in FamilyRepo::list_with_members(conn)? {
        for m in members { push(m, Some(family.name.clone())); }
    }
    for m in MemberRepo::list_unassigned(conn)? {
        push(m, None);
    }
    Ok(out)
}
