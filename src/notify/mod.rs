//! System notifications. Implemented for iOS; a no-op on other platforms for now.

/// One local notification to deliver at `fire_at` (unix seconds).
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledNotification {
    pub id:      String,
    pub title:   String,
    pub body:    String,
    pub fire_at: i64,
}

#[cfg(target_os = "ios")]
mod ios;
#[cfg(target_os = "ios")]
pub use ios::{clear_delivered, init, replace_all, request_permission};

/// Register the handler for the "Done for today" notification action. Call before launch.
#[cfg(not(target_os = "ios"))]
pub fn init(_on_done: fn(&str)) {}

/// Ask the user for permission to show notifications (no-op off iOS).
#[cfg(not(target_os = "ios"))]
pub fn request_permission() {}

/// Cancel all pending notifications of this app and schedule `items` (no-op off iOS).
#[cfg(not(target_os = "ios"))]
pub fn replace_all(_items: &[ScheduledNotification], _now: i64, _done_title: &str) {}

/// Remove already-delivered notifications by id (no-op off iOS).
#[cfg(not(target_os = "ios"))]
pub fn clear_delivered(_ids: &[String]) {}
