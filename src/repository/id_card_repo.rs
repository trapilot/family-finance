use rusqlite::Connection;
use std::collections::HashMap;

use crate::error::Result;
use crate::models::{IdCardImage, IdSide};

pub struct IdCardRepo;

impl IdCardRepo {
    /// Insert or replace one side's photo
    pub fn set(conn: &Connection, member_id: &str, side: IdSide, jpeg: &[u8]) -> Result<()> {
        conn.execute(
            "INSERT OR REPLACE INTO member_id_images (member_id, side, jpeg, updated_at)
             VALUES (?1,?2,?3,?4)",
            rusqlite::params![member_id, side.to_string(), jpeg, now_ts()],
        )?;
        Ok(())
    }

    pub fn remove(conn: &Connection, member_id: &str, side: IdSide) -> Result<()> {
        conn.execute(
            "DELETE FROM member_id_images WHERE member_id = ?1 AND side = ?2",
            rusqlite::params![member_id, side.to_string()],
        )?;
        Ok(())
    }

    pub fn get(conn: &Connection, member_id: &str, side: IdSide) -> Result<Option<Vec<u8>>> {
        let mut stmt = conn.prepare(
            "SELECT jpeg FROM member_id_images WHERE member_id = ?1 AND side = ?2",
        )?;
        match stmt.query_row(rusqlite::params![member_id, side.to_string()], |row| row.get(0)) {
            Ok(jpeg)                                  => Ok(Some(jpeg)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e)                                    => Err(e.into()),
        }
    }

    /// Which sides each member has, without loading the photos
    pub fn sides_by_member(conn: &Connection) -> Result<HashMap<String, Vec<IdSide>>> {
        let mut stmt = conn.prepare("SELECT member_id, side FROM member_id_images")?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?;
        let mut map: HashMap<String, Vec<IdSide>> = HashMap::new();
        for row in rows {
            let (member_id, side) = row?;
            if let Ok(side) = side.parse() {
                map.entry(member_id).or_default().push(side);
            }
        }
        Ok(map)
    }

    /// Every photo (for backups)
    pub fn list_all(conn: &Connection) -> Result<Vec<IdCardImage>> {
        let mut stmt = conn.prepare(
            "SELECT member_id, side, jpeg, updated_at FROM member_id_images",
        )?;
        let rows = stmt.query_map([], IdCardImage::from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
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
    use crate::models::NewMember;
    use crate::repository::MemberRepo;

    #[test]
    fn photos_round_trip_through_backup_json_and_go_with_the_member() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&conn).unwrap();
        let m = MemberRepo::create(&conn, &NewMember {
            family_id: None, full_name: "A".into(), birth_date: None, gender: None, phone: None,
            id_number: None, id_issue_date: None, id_issue_place: None, address: None,
            role: MemberRole::Member, avatar_emoji: None, note: None,
        }).unwrap();

        IdCardRepo::set(&conn, &m.id, IdSide::Front, &[0xFF, 0xD8, 1, 2]).unwrap();
        assert_eq!(IdCardRepo::get(&conn, &m.id, IdSide::Front).unwrap(), Some(vec![0xFF, 0xD8, 1, 2]));
        assert_eq!(IdCardRepo::get(&conn, &m.id, IdSide::Back).unwrap(), None);
        assert_eq!(IdCardRepo::sides_by_member(&conn).unwrap()[&m.id], [IdSide::Front]);

        let all = IdCardRepo::list_all(&conn).unwrap();
        let json = serde_json::to_string(&all).unwrap();
        assert!(json.contains("\"jpeg\":\"/9gBAg==\""));
        assert_eq!(serde_json::from_str::<Vec<IdCardImage>>(&json).unwrap(), all);

        MemberRepo::delete(&conn, &m.id).unwrap();
        assert!(IdCardRepo::list_all(&conn).unwrap().is_empty());
    }
}
