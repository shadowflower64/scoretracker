use crate::util::uuid::UuidString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Player {
    pub player_uuid: UuidString,
    pub name: String,
}

impl Player {
    pub fn from_postgres_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            player_uuid: row.try_get("player_uuid")?,
            name: row.try_get("name")?,
        })
    }
}
