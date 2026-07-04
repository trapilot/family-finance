use dioxus::prelude::*;
use crate::app::AppState;
use crate::repository::SettingsRepo;

#[component]
pub fn PinVerifyModal(
    is_visible: ReadSignal<bool>,
    on_cancel: EventHandler,
    on_verify: EventHandler,
) -> Element {
    let state = use_context::<AppState>();
    let mut pin = use_signal(|| String::new());
    let mut error = use_signal(|| None::<String>);

    // Clone state for each closure that uses it!
    let state_submit = state.clone();
    let state_keydown = state.clone();

    let on_submit = move |_: MouseEvent| {
        let p = pin.read().clone();
        
        let result = state_submit.with_conn(|conn| SettingsRepo::verify_pin(conn, &p));
        
        match result {
            Ok(true) => {
                pin.set(String::new());
                error.set(None);
                on_verify.call(());
            },
            Ok(false) => {
                error.set(Some("Incorrect PIN!".into()));
                pin.set(String::new());
            },
            Err(e) => {
                error.set(Some(format!("Error: {}", e)));
            }
        }
    };

    rsx! {
        if *is_visible.read() {
            div {
                style: "position:fixed; inset:0; background:rgba(0,0,0,.5); display:flex; align-items:center; justify-content:center; z-index:200;",
                div {
                    style: "background:#fff; border-radius:16px; padding:24px; margin:16px; max-width:320px; width:100%;",
                    h3 { style: "margin:0 0 8px; font-size:18px; color:#1f2937; text-align:center;", "Enter PIN" },
                    p { style: "margin:0 0 20px; font-size:14px; color:#6b7280; text-align:center;", "Please enter your PIN to continue." },

                    if let Some(err) = error.read().clone() {
                        div { style: "width:100%; background:#fee2e2; border:1px solid #ef4444; border-radius:8px; padding:10px; color:#ef4444; text-align:center; margin-bottom:16px;", "{err}" }
                    }

                    input {
                        r#type: "password",
                        inputmode: "numeric",
                        maxlength: 6,
                        style: "width:100%; padding:12px; border:1px solid #e5e7eb; border-radius:8px; font-size:16px; text-align:center; letter-spacing:4px; margin-bottom:16px;",
                        value: "{pin.read().clone()}",
                        oninput: move |e| {
                            let v = e.value().chars().filter(|c| c.is_ascii_digit()).collect();
                            pin.set(v);
                            error.set(None);
                        },
                        onkeydown: move |e| {
                            if e.key() == Key::Enter {
                                // Reuse verification logic without needing Event
                                let p = pin.read().clone();
                                let result = state_keydown.with_conn(|conn| SettingsRepo::verify_pin(conn, &p));
                                match result {
                                    Ok(true) => {
                                        pin.set(String::new());
                                        error.set(None);
                                        on_verify.call(());
                                    },
                                    Ok(false) => {
                                        error.set(Some("Incorrect PIN!".into()));
                                        pin.set(String::new());
                                    },
                                    Err(e) => {
                                        error.set(Some(format!("Error: {}", e)));
                                    }
                                }
                            }
                        }
                    }

                    div { style: "display:flex; gap:10px;",
                        button {
                            style: "flex:1; padding:11px; border:1px solid #e5e7eb; border-radius:8px; background:#fff; cursor:pointer; font-size:14px;",
                            onclick: move |_| {
                                pin.set(String::new());
                                error.set(None);
                                on_cancel.call(());
                            },
                            "Cancel"
                        }
                        button {
                            style: "flex:1; padding:11px; border:none; border-radius:8px; background:#6366f1; color:#fff; cursor:pointer; font-size:14px; font-weight:700;",
                            onclick: on_submit,
                            "Verify"
                        }
                    }
                }
            }
        }
    }
}
