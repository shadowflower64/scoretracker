use crate::data::scoreboard::metadata::ArbitraryMetadata;
use crate::util::timestamp::{NsDuration, NsTimestamp};
use crate::util::{command_line::AskError, uuid::UuidString};
use dyn_clone::{DynClone, clone_trait_object};
use postgres::types::FromSql;
use schemars::{JsonSchema, json_schema};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::Debug;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Match {
    /// UUID of the match.
    pub match_uuid: UuidString,

    /// Timestamp of the match - specifically, the timestamp of the first frame of the end screen. Can be approximate.
    pub timestamp: NsTimestamp,

    /// Named ID of the chartset.
    pub chartset_id: String,

    /// List of library entry UUIDs that are proof of this match.
    pub proof: Vec<UuidString>,

    /// Game-specific details of the match.
    pub details: Box<AnyMatchDetails>,

    /// Any additional match metadata.
    pub metadata: ArbitraryMetadata,
}

impl Match {
    pub fn from_postgres_row(row: &postgres::Row) -> Result<Self, postgres::Error> {
        Ok(Self {
            match_uuid: row.try_get("match_uuid")?,
            timestamp: row.try_get("timestamp")?,
            chartset_id: row.try_get("chartset_id")?,
            proof: row.try_get("proof")?,
            details: row.try_get("details")?,
            metadata: row.try_get("metadata")?,
        })
    }
}

#[typetag::serde(tag = "game")]
pub trait MatchDetails: Debug + DynClone {
    fn game_id(&self) -> &'static str {
        self.typetag_name()
    }
    fn ask_for_match_edit(&mut self) -> Result<(), AskError> {
        unimplemented!()
    }
    fn sorting_key(&self) -> f64;
    fn check_vitals(&self) -> Result<(), String> {
        Ok(())
    }
}
clone_trait_object! {MatchDetails}
pub type AnyMatchDetails = dyn MatchDetails + 'static;

impl<'a> FromSql<'a> for Box<AnyMatchDetails> {
    fn from_sql(ty: &postgres::types::Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let value = serde_json::Value::from_sql(ty, raw)?;
        let details = serde_json::from_value(value)?;
        Ok(details)
    }

    fn accepts(ty: &postgres::types::Type) -> bool {
        serde_json::Value::accepts(ty)
    }
}

impl JsonSchema for Box<AnyMatchDetails> {
    fn schema_name() -> Cow<'static, str> {
        "AnyMatchDetails".into()
    }

    fn json_schema(_gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
        json_schema!({
            "type": "object"
        })
    }
}

pub const ADD_TOO_CLOSE_THRESHOLD_SECONDS: f64 = 60.0;

#[derive(Debug, Error)]
pub enum MatchInsertError {
    #[error("match is too close to an existing match: {0} (time difference: {1})")]
    TooClose(Uuid, NsDuration),
    #[error("match is already in the database: {0}")]
    ExistsAlready(Uuid),
}

// pub fn find_other_close_matches(
//     &self,
//     req_m: &dyn MatchDetails,
//     threshold: NsDuration,
// ) -> Vec<(&(dyn MatchDetails + 'static), NsDuration)> {
//     let mut search_results = self
//         .matches
//         .iter()
//         .filter_map(|m| {
//             let difference = (m.timestamp() - req_m.timestamp()).abs();
//             (m.uuid() != req_m.uuid() && m.game_id() == req_m.game_id() && difference <= threshold).then_some((m.as_ref(), difference))
//         })
//         .collect::<Vec<_>>();
//     search_results.sort_by_key(|(_, how_close)| *how_close);
//     search_results
// }

// pub fn insert_new(&mut self, match_data: Match) -> Result<Uuid, MatchInsertError> {
//     let threshold = NsDuration::from_secs_f64(Self::ADD_TOO_CLOSE_THRESHOLD_SECONDS);
//     if let Some((close_match, how_close)) = self.find_other_close_matches(match_data.as_ref(), threshold).first() {
//         return Err(MatchInsertError::TooClose(close_match.uuid().0, *how_close));
//     }

//     if let Some(existing_match) = self.find_match_by_uuid(match_data.match_uuid()) {
//         return Err(MatchInsertError::ExistsAlready(existing_match.uuid().0));
//     }

//     let uuid = match_data.match_uuid();
//     self.matches.push(match_data);
//     Ok(uuid.0)
// }
