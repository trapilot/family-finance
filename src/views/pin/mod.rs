use dioxus::prelude::*;
use crate::i18n::use_i18n;
use crate::app::AppState;
use crate::repository::SettingsRepo;

pub mod verify;
pub use verify::PinVerifyModal;

#[component]
pub fn PinSetup(on_set: EventHandler) -> Element {
    let t = use_i18n();
    let state = use_context::<AppState>();
    let mut pin = use_signal(|| String::new());
    let mut confirm_pin = use_signal(|| String::new());
    let mut error = use_signal(|| None::<String>);

    let state_clone = state.clone();
    let mut on_submit = move || {
        let p = pin.read().clone();
        let cp = confirm_pin.read().clone();
        
        if p.len() < 4 {
            error.set(Some(t.t("PIN must be at least 4 digits!").into()));
            return;
        }
        
        if p != cp {
            error.set(Some(t.t("PINs do not match!").into()));
            return;
        }
        
        let result = state_clone.with_conn(|conn| SettingsRepo::set_pin(conn, &p));
        
        match result {
            Ok(_) => {
                on_set.call(());
            },
            Err(e) => {
                error.set(Some(format!("{}: {}", t.t("Failed to set PIN"), e)));
            }
        }
    };

    rsx! {
        div {
            style: "display:flex; flex-direction:column; gap:20px; padding:calc(40px + env(safe-area-inset-top)) 20px calc(40px + env(safe-area-inset-bottom)); align-items:center; position:fixed; top:0; right:0; bottom:0; left:0; overflow-y:auto; justify-content:center; font-family:system-ui,sans-serif;",
            div { style: "font-size:40px;", "🔐" }
            h1 { style: "font-size:24px; font-weight:700; color:#1f2937;", {t.t("Set your PIN")} },
            p { style: "font-size:14px; color:#6b7280; text-align:center;", {t.t("You will need this PIN to unlock the app and access sensitive features")} },
            
            if let Some(err) = error.read().clone() {
                div {
                    style: "width:100%; max-width:300px; background:#fee2e2; border:2px solid #ef4444; border-radius:8px; padding:14px; color:#ef4444; font-weight:700; text-align:center; font-size:14px;",
                    "{err}"
                }
            }
            
            form {
                style: "display:flex; flex-direction:column; gap:16px; width:100%; max-width:300px;",
                onsubmit: move |e| {
                    e.prevent_default();
                    on_submit();
                },
                div { style: "display:flex; flex-direction:column; gap:8px;",
                    label { style: "font-weight:600; font-size:14px; color:#374151;", {t.t("Enter PIN")} },
                    input {
                        r#type: "password",
                        inputmode: "numeric",
                        enterkeyhint: "done",
                        autocomplete: "off",
                        maxlength: 6,
                        style: "width:100%; padding:12px; border:1px solid #e5e7eb; border-radius:8px; font-size:16px; text-align:center; letter-spacing:4px;",
                        value: "{pin.read().clone()}",
                        oninput: move |e| {
                            let v = e.value().chars().filter(|c| c.is_ascii_digit()).collect();
                            pin.set(v);
                            error.set(None);
                        }
                    }
                }
                
                div { style: "display:flex; flex-direction:column; gap:8px;",
                    label { style: "font-weight:600; font-size:14px; color:#374151;", {t.t("Confirm PIN")} },
                    input {
                        r#type: "password",
                        inputmode: "numeric",
                        enterkeyhint: "done",
                        autocomplete: "off",
                        maxlength: 6,
                        style: "width:100%; padding:12px; border:1px solid #e5e7eb; border-radius:8px; font-size:16px; text-align:center; letter-spacing:4px;",
                        value: "{confirm_pin.read().clone()}",
                        oninput: move |e| {
                            let v = e.value().chars().filter(|c| c.is_ascii_digit()).collect();
                            confirm_pin.set(v);
                            error.set(None);
                        }
                    }
                }
                
                button {
                    style: "margin-top:16px; width:100%; padding:14px; background:#6366f1; color:#fff; border:none; border-radius:12px; font-weight:700; font-size:16px; cursor:pointer;",
                    r#type: "submit",
                    {t.t("Continue")}
                }
            }
        }
    }
}

#[component]
pub fn PinUnlock(on_unlock: EventHandler) -> Element {
    let t = use_i18n();
    let state = use_context::<AppState>();
    let mut pin = use_signal(|| String::new());
    let mut error = use_signal(|| None::<String>);

    let mut on_submit = move || {
        let p = pin.read().clone();
        
        let result = state.with_conn(|conn| SettingsRepo::verify_pin(conn, &p));
        
        match result {
            Ok(true) => {
                on_unlock.call(());
            },
            Ok(false) => {
                error.set(Some(t.t("Incorrect PIN!").into()));
                pin.set(String::new());
            },
            Err(e) => {
                error.set(Some(format!("{}: {}", t.t("Error"), e)));
            }
        }
    };

    rsx! {
        div {
            style: "display:flex; flex-direction:column; gap:20px; padding:calc(40px + env(safe-area-inset-top)) 20px calc(40px + env(safe-area-inset-bottom)); align-items:center; position:fixed; top:0; right:0; bottom:0; left:0; overflow-y:auto; justify-content:center; font-family:system-ui,sans-serif;",
            div { style: "font-size:40px;", "🔒" }
            h1 { style: "font-size:24px; font-weight:700; color:#1f2937;", {t.t("Enter your PIN")} },
            
            if let Some(err) = error.read().clone() {
                div {
                    style: "width:100%; max-width:300px; background:#fee2e2; border:2px solid #ef4444; border-radius:8px; padding:14px; color:#ef4444; font-weight:700; text-align:center; font-size:14px;",
                    "{err}"
                }
            }
            
            form {
                style: "display:flex; flex-direction:column; gap:16px; width:100%; max-width:300px;",
                onsubmit: move |e| {
                    e.prevent_default();
                    on_submit();
                },
                div { style: "display:flex; flex-direction:column; gap:8px;",
                    input {
                        r#type: "password",
                        inputmode: "numeric",
                        enterkeyhint: "done",
                        autocomplete: "off",
                        maxlength: 6,
                        style: "width:100%; padding:12px; border:1px solid #e5e7eb; border-radius:8px; font-size:16px; text-align:center; letter-spacing:4px;",
                        value: "{pin.read().clone()}",
                        oninput: move |e| {
                            let v = e.value().chars().filter(|c| c.is_ascii_digit()).collect();
                            pin.set(v);
                            error.set(None);
                        }
                    }
                }
                
                button {
                    style: "margin-top:16px; width:100%; padding:14px; background:#6366f1; color:#fff; border:none; border-radius:12px; font-weight:700; font-size:16px; cursor:pointer;",
                    r#type: "submit",
                    {t.t("Unlock")}
                }
            }
        }
    }
}
