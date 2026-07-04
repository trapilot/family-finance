// Phase 2 — Members is a pure contact directory, no transaction linking.
// Business logic is minimal; views call MemberRepo directly.
// This file is intentionally thin — kept for crate::services consistency.

use rusqlite::Connection;
use crate::error::Result;
use crate::models::Member;
use crate::repository::MemberRepo;

pub struct MemberService;

impl MemberService {
    /// Returns members sorted: owners first, then alphabetically.
    pub fn list(conn: &Connection) -> Result<Vec<Member>> {
        MemberRepo::list(conn)
    }
}
