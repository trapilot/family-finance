//! System notifications. Implemented for iOS; a no-op on other platforms for now.

/// One local notification to deliver at `fire_at` (unix seconds).
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledNotification {
    pub id:       String,
    pub title:    String,
    pub body:     String,
    pub fire_at:  i64,
    pub category: Category,
}

/// Decides the notification's button: "Done for today" or "Paid this month"
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Category {
    Reminder,
    Bill,
}

/// Button titles for each category, in the user's language
#[cfg_attr(not(target_os = "ios"), allow(dead_code))]
pub struct ActionTitles<'a> {
    pub done_for_today:  &'a str,
    pub paid_this_month: &'a str,
}

#[cfg(target_os = "ios")]
mod ios;
#[cfg(target_os = "ios")]
pub use ios::{clear_delivered, init, replace_all, request_permission};

/// Register the handler for the "Done for today" / "Paid this month" notification action. Call before launch.
#[cfg(not(target_os = "ios"))]
pub fn init(_on_done: fn(&str)) {}

/// Ask the user for permission to show notifications (no-op off iOS).
#[cfg(not(target_os = "ios"))]
pub fn request_permission() {}

/// Cancel all pending notifications of this app and schedule `items` (no-op off iOS).
#[cfg(not(target_os = "ios"))]
pub fn replace_all(_items: &[ScheduledNotification], _now: i64, _titles: &ActionTitles) {}

/// Remove already-delivered notifications by id (no-op off iOS).
#[cfg(not(target_os = "ios"))]
pub fn clear_delivered(_ids: &[String]) {}
