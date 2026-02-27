use serde::{Deserialize, Serialize};

pub mod r#match;
pub mod performance;
pub mod score_db;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum AnyValue {
    String(String),
    Number(f64),
    Bool(bool),
}
