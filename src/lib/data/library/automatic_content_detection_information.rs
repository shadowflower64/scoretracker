use std::error::Error;

use postgres_types::{FromSql, IsNull, ToSql, to_sql_checked};
use serde::{Deserialize, Serialize};

use crate::util::timestamp::NsTimestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guess<T> {
    prediction: T,
    confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction<T> {
    program_identifier: String,
    timestamp: NsTimestamp,
    guesses: Vec<Guess<T>>,
}

pub type AutomaticallyDetected<T> = Vec<Prediction<T>>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutomaticContentDetectionInformation {
    song_title: AutomaticallyDetected<String>,
    song_artist: AutomaticallyDetected<String>,
    song_id: AutomaticallyDetected<String>,
    player_name: AutomaticallyDetected<String>,
    instrument: AutomaticallyDetected<String>,
    difficulty: AutomaticallyDetected<String>,
    score: AutomaticallyDetected<f64>,
    note_streak: AutomaticallyDetected<u64>,
    note_hits: AutomaticallyDetected<u64>,
    notes_total: AutomaticallyDetected<u64>,
}

impl<'a> FromSql<'a> for AutomaticContentDetectionInformation {
    fn accepts(ty: &postgres_types::Type) -> bool {
        <serde_json::Value as FromSql>::accepts(ty)
    }
    fn from_sql(ty: &postgres_types::Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let value = serde_json::Value::from_sql(ty, raw)?;
        Ok(serde_json::from_value(value)?)
    }
}

impl ToSql for AutomaticContentDetectionInformation {
    fn accepts(ty: &postgres_types::Type) -> bool
    where
        Self: Sized,
    {
        <serde_json::Value as ToSql>::accepts(ty)
    }
    fn to_sql(&self, ty: &postgres_types::Type, out: &mut actix_web::web::BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>>
    where
        Self: Sized,
    {
        serde_json::value::to_value(self.clone())?.to_sql(ty, out)
    }
    to_sql_checked! {}
}
