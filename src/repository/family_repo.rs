use rusqlite::Connection;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::family::{Family, NewFamily};
use crate::models::Member;

pub struct FamilyRepo;

impl FamilyRepo {
    const SELECT_COLS: &'static str =
        "id, name, common_address, note, created_at";

    pub fn create(conn: &Connection, input: &NewFamily) -> Result<Family> {
        let id  = Uuid::new_v4().to_string();
        let now = now_ts();

        conn.execute(
            "INSERT INTO families
             (id, name, common_address, note, created_at)
             VALUES (?1,?2,?3,?4,?5)",
            rusqlite::params![
                id,
                input.name,
                input.common_address,
                input.note,
                now,
            ],
        )?;

        Self::find_by_id(conn, &id)?.ok_or_else(|| AppError::NotFound(id))
    }

    pub fn find_by_id(conn: &Connection, id: &str) -> Result<Option<Family>> {
        let sql = format!("SELECT {} FROM families WHERE id = ?1", Self::SELECT_COLS);
        let mut stmt = conn.prepare(&sql)?;
        match stmt.query_row([id], Family::from_row) {
            Ok(f)                                     => Ok(Some(f)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e)                                    => Err(e.into()),
        }
    }

    pub fn list(conn: &Connection) -> Result<Vec<Family>> {
        let sql = format!(
            "SELECT {} FROM families ORDER BY name ASC",
            Self::SELECT_COLS
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], Family::from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
    }

    pub fn list_with_members(conn: &Connection) -> Result<Vec<(Family, Vec<Member>)>> {
        let families = Self::list(conn)?;
        let members = crate::repository::MemberRepo::list(conn)?;
        
        let mut result = Vec::new();
        for family in families {
            let family_members = members
                .iter()
                .filter(|m| m.family_id.as_deref() == Some(&family.id))
                .cloned()
                .collect();
            result.push((family, family_members));
        }
        
        Ok(result)
    }

    pub fn update(conn: &Connection, family: &Family) -> Result<()> {
        conn.execute(
            "UPDATE families
             SET name = ?1, common_address = ?2, note = ?3
             WHERE id = ?4",
            rusqlite::params![
                family.name,
                family.common_address,
                family.note,
                family.id,
            ],
        )?;
        Ok(())
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<()> {
        // Unlink members first (set family_id = NULL)
        conn.execute(
            "UPDATE members SET family_id = NULL WHERE family_id = ?1",
            [id],
        )?;
        let affected = conn.execute("DELETE FROM families WHERE id = ?1", [id])?;
        if affected == 0 {
            return Err(AppError::NotFound(format!("family '{id}'")));
        }
        Ok(())
    }
}

fn now_ts() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}
