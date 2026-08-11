use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod r#match;
pub mod performance;
pub mod player;
pub mod score_db;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum MetadataValue {
    String(String),
    Number(f64),
    Bool(bool),
}

pub type SongId = String;
