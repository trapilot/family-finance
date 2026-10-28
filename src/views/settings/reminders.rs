use chrono::Local;
use dioxus::prelude::*;

use crate::app::AppState;
use crate::i18n::use_i18n;
use crate::models::{Reminder, Wallet};
use crate::repository::settings_repo::parse_hh_mm;
use crate::repository::{ReminderRepo, SettingsRepo, WalletRepo};
use crate::services::ReminderService;
use crate::views::{repeat_unit_label, ts_to_date_str};

#[component]
pub fn RemindersSettings() -> Element {
    let t = use_i18n();
    let state = use_context::<AppState>();

    let mut reminders = use_signal(Vec::<Reminder>::new);
    let mut wallets   = use_signal(Vec::<Wallet>::new);
    let mut interval  = use_signal(|| "4".to_string());
    let mut start     = use_signal(|| "08:00".to_string());

    let reload = {
        let state = state.clone();
        move || {
            state.with_conn(|conn| {
                reminders.set(ReminderRepo::list(conn).unwrap_or_default());
                wallets.set(WalletRepo::list_active(conn).unwrap_or_default());
                interval.set(SettingsRepo::get_reminder_interval_hours(conn).unwrap_or(4).to_string());
                let (h, m) = SettingsRepo::get_reminder_start_time(conn).unwrap_or((8, 0));
                start.set(format!("{h:02}:{m:02}"));
            });
        }
    };
    use_hook({
        let mut reload = reload.clone();
        move || reload()
    });

    let save_interval = {
        let state = state.clone();
        let mut reload = reload.clone();
        move |_| {
            let hours = interval.read().parse::<u32>().unwrap_or(4);
            let _ = state.with_conn(|conn| {
                SettingsRepo::set_reminder_interval_hours(conn, hours)?;
                ReminderService::resync(conn)
            });
            reload();
        }
    };

    let save_start = {
        let state = state.clone();
        let mut reload = reload.clone();
        move |e: FormEvent| {
            if let Some((h, m)) = parse_hh_mm(&e.value()) {
                let _ = state.with_conn(|conn| {
                    SettingsRepo::set_reminder_start_time(conn, h, m)?;
                    ReminderService::resync(conn)
                });
            }
            reload();
        }
    };

    let today = Local::now().date_naive();

    rsx! {
        div { style: "display:flex; flex-direction:column; gap:16px;",

            // ── Interval ──
            div {
                style: "background:#fff; border-radius:12px; padding:16px; box-shadow:0 1px 3px rgba(0,0,0,.06);",
                h2 { style: "margin:0 0 6px; font-size:16px; font-weight:700; color:#1f2937;", {t.t("Notification frequency")} }
                p { style: "margin:0 0 12px; font-size:13px; color:#6b7280; line-height:1.5;",
                    {t.t("On a day with due income or transfers, notify from the first notification time, then every N hours until midnight.")}
                }
                div { style: "display:flex; align-items:center; gap:8px; font-size:14px; color:#374151; margin-bottom:10px;",
                    {t.t("First notification at")}
                    input {
                        style: "border:1px solid #e5e7eb; border-radius:8px; padding:8px 10px; font-size:16px; outline:none; background:#fff;",
                        r#type: "time",
                        value: "{start.read()}",
                        oninput: move |e| start.set(e.value()),
                        onchange: save_start,
                    }
                }
                div { style: "display:flex; align-items:center; gap:8px; font-size:14px; color:#374151;",
                    {t.t("Every")}
                    input {
                        style: "width:64px; border:1px solid #e5e7eb; border-radius:8px; padding:8px 10px; font-size:16px; outline:none; background:#fff; text-align:center;",
                        r#type: "text",
                        inputmode: "numeric",
                        value: "{interval.read()}",
                        oninput: move |e| interval.set(e.value().chars().filter(|c| c.is_ascii_digit()).take(2).collect()),
                        onchange: save_interval,
                    }
                    {t.t("hours (1–24)")}
                }
            }

            // ── List ──
            div {
                style: "background:#fff; border-radius:12px; padding:16px; box-shadow:0 1px 3px rgba(0,0,0,.06); display:flex; flex-direction:column; gap:12px;",
                h2 { style: "margin:0; font-size:16px; font-weight:700; color:#1f2937;", {t.t("Recurring reminders")} }
                if reminders.read().is_empty() {
                    p { style: "margin:0; font-size:13px; color:#9ca3af; line-height:1.5;",
                        {t.t("No reminders yet. Turn on \"Remind me when this repeats\" when adding an Income or Transfer.")}
                    }
                }
                for reminder in reminders.read().iter().cloned() {
                    {
                        let (title, body) = ReminderService::describe(&reminder, &wallets.read(), t.lang());
                        let schedule = t.t("Every {n} {unit} · from {date}")
                            .replace("{n}", &reminder.repeat_every.to_string())
                            .replace("{unit}", t.t(repeat_unit_label(reminder.repeat_unit)))
                            .replace("{date}", &ts_to_date_str(reminder.start_date));
                        let next = reminder.next_due(today)
                            .map(|d| d.format("%d/%m/%Y").to_string())
                            .unwrap_or_else(|| "—".into());
                        let paused = reminder.is_paused;
                        let state_pause = state.clone();
                        let mut reload_pause = reload.clone();
                        let id_pause = reminder.id.clone();
                        let state = state.clone();
                        let mut reload = reload.clone();
                        let id = reminder.id.clone();
                        rsx! {
                            div {
                                key: "{reminder.id}",
                                style: "display:flex; gap:10px; align-items:flex-start; border-top:1px solid #f3f4f6; padding-top:12px;",
                                div { style: "flex:1; min-width:0;",
                                    div { style: "font-size:13px; font-weight:700; color:#1f2937;", "{title}" }
                                    div { style: "font-size:13px; color:#374151; overflow-wrap:anywhere;", "{body}" }
                                    div { style: "font-size:12px; color:#6b7280; margin-top:2px;", "{schedule}" }
                                    if paused {
                                        div { style: "font-size:12px; color:#f59e0b; margin-top:2px; font-weight:600;", {t.t("⏸ Paused — no notifications")} }
                                    } else {
                                        div { style: "font-size:12px; color:#6366f1; margin-top:2px;", {t.t("Next")} ": {next}" }
                                    }
                                }
                                button {
                                    style: "border:none; background:#f3f4f6; color:#374151; border-radius:8px; padding:6px 10px; font-size:13px; cursor:pointer; flex-shrink:0;",
                                    onclick: move |_| {
                                        let _ = state_pause.with_conn(|conn| {
                                            ReminderRepo::set_paused(conn, &id_pause, !paused)?;
                                            ReminderService::resync(conn)
                                        });
                                        reload_pause();
                                    },
                                    if paused { {t.t("▶ Resume")} } else { {t.t("⏸ Pause")} }
                                }
                                button {
                                    style: "border:none; background:#fef2f2; color:#ef4444; border-radius:8px; padding:6px 10px; font-size:13px; cursor:pointer; flex-shrink:0;",
                                    onclick: move |_| {
                                        let _ = state.with_conn(|conn| {
                                            ReminderRepo::delete(conn, &id)?;
                                            ReminderService::resync(conn)
                                        });
                                        reload();
                                    },
                                    {t.t("Delete")}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
