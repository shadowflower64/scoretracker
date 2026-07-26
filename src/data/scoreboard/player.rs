use crate::util::uuid::UuidString;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Player {
    pub uuid: UuidString,
    pub name: String,
}
