use rusqlite::Row;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Family {
    pub id: String,
    pub name: String,
    pub common_address: Option<String>,
    pub note: Option<String>,
    pub created_at: i64,
}

impl Family {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Family {
            id: row.get("id")?,
            name: row.get("name")?,
            common_address: row.get("common_address")?,
            note: row.get("note")?,
            created_at: row.get("created_at")?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewFamily {
    pub name: String,
    pub common_address: Option<String>,
    pub note: Option<String>,
}
