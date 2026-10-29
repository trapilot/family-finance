use rusqlite::Connection;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::member::{Member, NewMember};

pub struct MemberRepo;

impl MemberRepo {
    const SELECT_COLS: &'static str =
        "id, family_id, full_name, birth_date, gender, phone, id_number,
         id_issue_date, id_issue_place, address, role, avatar_emoji, note, created_at";

    /// Owners first, then by date of birth (oldest first, unknown last), then name
    const ORDER: &'static str =
        "CASE WHEN role = 'owner' THEN 0 ELSE 1 END, birth_date IS NULL, birth_date ASC, full_name ASC";

    pub fn create(conn: &Connection, input: &NewMember) -> Result<Member> {
        let id  = Uuid::new_v4().to_string();
        let now = now_ts();

        conn.execute(
            "INSERT INTO members
             (id, family_id, full_name, birth_date, gender, phone, id_number,
              id_issue_date, id_issue_place, address, role, avatar_emoji, note, created_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
            rusqlite::params![
                id,
                input.family_id,
                input.full_name,
                input.birth_date,
                input.gender.as_ref().map(|g| g.to_string()),
                input.phone,
                input.id_number,
                input.id_issue_date,
                input.id_issue_place,
                input.address,
                input.role.to_string(),
                input.avatar_emoji,
                input.note,
                now,
            ],
        )?;

        Self::find_by_id(conn, &id)?.ok_or_else(|| AppError::NotFound(id))
    }

    pub fn find_by_id(conn: &Connection, id: &str) -> Result<Option<Member>> {
        let sql = format!("SELECT {} FROM members WHERE id = ?1", Self::SELECT_COLS);
        let mut stmt = conn.prepare(&sql)?;
        match stmt.query_row([id], Member::from_row) {
            Ok(m)                                     => Ok(Some(m)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e)                                    => Err(e.into()),
        }
    }

    pub fn list(conn: &Connection) -> Result<Vec<Member>> {
        let sql = format!(
            "SELECT {} FROM members ORDER BY {}",
            Self::SELECT_COLS, Self::ORDER
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], Member::from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
    }

    pub fn list_by_family(conn: &Connection, family_id: &str) -> Result<Vec<Member>> {
        let sql = format!(
            "SELECT {} FROM members WHERE family_id = ?1 ORDER BY {}",
            Self::SELECT_COLS, Self::ORDER
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([family_id], Member::from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
    }

    pub fn list_unassigned(conn: &Connection) -> Result<Vec<Member>> {
        let sql = format!(
            "SELECT {} FROM members WHERE family_id IS NULL ORDER BY {}",
            Self::SELECT_COLS, Self::ORDER
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], Member::from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
    }

    pub fn update(conn: &Connection, member: &Member) -> Result<()> {
        conn.execute(
            "UPDATE members
             SET family_id = ?1, full_name = ?2, birth_date = ?3, gender = ?4, phone = ?5,
                 id_number = ?6, id_issue_date = ?7, id_issue_place = ?8,
                 address = ?9, role = ?10, avatar_emoji = ?11, note = ?12
             WHERE id = ?13",
            rusqlite::params![
                member.family_id,
                member.full_name,
                member.birth_date,
                member.gender.as_ref().map(|g| g.to_string()),
                member.phone,
                member.id_number,
                member.id_issue_date,
                member.id_issue_place,
                member.address,
                member.role.to_string(),
                member.avatar_emoji,
                member.note,
                member.id,
            ],
        )?;
        Ok(())
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<()> {
        // (transactions have no member_id column yet, so there is nothing to unlink)
        conn.execute("DELETE FROM member_id_images WHERE member_id = ?1", [id])?;
        let affected = conn.execute("DELETE FROM members WHERE id = ?1", [id])?;
        if affected == 0 {
            return Err(AppError::NotFound(format!("member '{id}'")));
        }
        Ok(())
    }

    /// How many transactions are tagged to this member
    pub fn transaction_count(conn: &Connection, member_id: &str) -> Result<i64> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transactions WHERE member_id = ?1",
            [member_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }
}

fn now_ts() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::member::MemberRole;

    fn add(conn: &Connection, name: &str, role: MemberRole, birth_date: Option<i64>) {
        MemberRepo::create(conn, &NewMember {
            family_id: None, full_name: name.into(), birth_date, gender: None, phone: None,
            id_number: None, id_issue_date: None, id_issue_place: None, address: None,
            role, avatar_emoji: None, note: None,
        }).unwrap();
    }

    #[test]
    fn owners_first_then_oldest_first_unknown_birth_last() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&conn).unwrap();
        add(&conn, "Kid",        MemberRole::Member, Some(1_400_000_000));
        add(&conn, "No DOB",     MemberRole::Member, None);
        add(&conn, "Grandma",    MemberRole::Member, Some(-300_000_000));
        add(&conn, "Owner",      MemberRole::Owner,  Some(600_000_000));
        add(&conn, "Mom",        MemberRole::Member, Some(700_000_000));

        let names: Vec<String> = MemberRepo::list_unassigned(&conn).unwrap().into_iter().map(|m| m.full_name).collect();
        assert_eq!(names, ["Owner", "Grandma", "Mom", "Kid", "No DOB"]);
    }
}
