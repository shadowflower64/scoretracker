use crate::data::scoreboard::metadata::ArbitraryMetadata;
use crate::util::timestamp::NsDuration;
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
pub struct Performance {
    /// UUID of the performance.
    pub performance_uuid: UuidString,

    /// Player UUID.
    pub player_uuid: UuidString,

    /// Match UUID.
    pub match_uuid: UuidString,

    /// Game ID. Has to match the game ID in the referenced match.
    pub game: String,

    /// Chartset ID. Has to match the chartset ID in the referenced match.
    pub chartset_id: String,

    /// Snake-case name of the instrument/play mode of the played chart.
    ///
    /// Examples:
    /// * for osu! this is either `standard`, `taiko`, `mania`, or `catch`.
    /// * for Guitar Hero this is either `guitar`, `bass`, `drums`, or `vocals`.
    /// * for DJMAX RESPECT V this is `4k`, `5k`, `6k`, `8k`, `4b`, `5b`, `6b`, or `8b`.
    /// * for Duolingo Music this is always `piano`.
    ///
    /// etc.
    pub instrument: String,

    /// Snake-case name of the difficulty of the played chart. (`easy`, `hard`, `insane`, etc...)
    pub difficulty: String,

    /// List of library entry UUIDs that are proof of this performance.
    pub proofs: Vec<UuidString>,

    /// Game-specific details of the performance.
    pub details: Box<AnyPerformanceDetails>,

    /// Any additional performance metadata.
    pub metadata: ArbitraryMetadata,
}

impl Performance {
    pub fn from_postgres_row(row: &postgres::Row) -> Result<Self, postgres::Error> {
        Ok(Self {
            performance_uuid: row.try_get("performance_uuid")?,
            player_uuid: row.try_get("player_uuid")?,
            match_uuid: row.try_get("match_uuid")?,
            game: row.try_get("game")?,
            chartset_id: row.try_get("chartset_id")?,
            instrument: row.try_get("instrument")?,
            difficulty: row.try_get("difficulty")?,
            proofs: row.try_get("proofs")?,
            details: row.try_get("details")?,
            metadata: row.try_get("metadata")?,
        })
    }
}

#[typetag::serde(tag = "game")]
pub trait PerformanceDetails: Debug + DynClone {
    fn game_id(&self) -> &'static str {
        self.typetag_name()
    }
    fn ask_for_performance_edit(&mut self) -> Result<(), AskError> {
        unimplemented!()
    }
    fn sorting_key(&self) -> f64;
    fn check_vitals(&self) -> Result<(), String> {
        Ok(())
    }
}

clone_trait_object! {PerformanceDetails}
pub type AnyPerformanceDetails = dyn PerformanceDetails + 'static;

impl<'a> FromSql<'a> for Box<AnyPerformanceDetails> {
    fn from_sql(ty: &postgres::types::Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let value = serde_json::Value::from_sql(ty, raw)?;
        let details = serde_json::from_value(value)?;
        Ok(details)
    }

    fn accepts(ty: &postgres::types::Type) -> bool {
        serde_json::Value::accepts(ty)
    }
}

impl JsonSchema for Box<AnyPerformanceDetails> {
    fn schema_name() -> Cow<'static, str> {
        "AnyPerformanceDetails".into()
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        json_schema!({
            "type": "object"
        })
    }
}

#[derive(Debug, Error)]
pub enum PerformanceInsertError {
    #[error("performance is too close to an existing performance: {0} (time difference: {1})")]
    TooClose(Uuid, NsDuration),
    #[error("performance is already in the database: {0}")]
    ExistsAlready(Uuid),
    #[error("match uuid referenced by performance is not present in the match database: {0}")]
    MatchIsNotInDatabase(Uuid),
}

// pub fn find_close_performances_from_diff_match(
//     &self,
//     req_p: &dyn PerformanceDetails,
//     threshold: NsDuration,
//     match_db: &MatchDatabase,
// ) -> Result<Vec<(&dyn PerformanceDetails, NsDuration)>, Uuid> {
//     let req_match_uuid = req_p.match_uuid();
//     let req_m = match_db.find_match_by_uuid(req_match_uuid).ok_or(req_match_uuid)?; // TODO: optimize this away
//     let req_timestamp = req_m.timestamp();
//     let mut search_results: Vec<(&dyn PerformanceDetails, NsDuration)> = self
//         .performances
//         .iter()
//         .filter_map(|ent| {
//             if ent.cached_timestamp.get().is_none() {
//                 ent.cached_timestamp.set({
//                     let m = match_db
//                         .find_match_by_uuid(ent.match_uuid())
//                         .expect("todo: performance's match data not found - database is in an invalid state");
//                     Some(m.timestamp())
//                 });
//             }
//             let p_timestamp = ent.cached_timestamp.get().expect("the code above should set the value to Some");

//             let difference = (p_timestamp - req_timestamp).abs();
//             (ent.match_uuid() != req_match_uuid && ent.game_id() == req_p.game_id() && difference <= threshold)
//                 .then_some((ent.as_ref(), difference))
//         })
//         .collect();
//     search_results.sort_by_key(|(_, how_close)| *how_close);
//     Ok(search_results)
// }

// pub fn insert_new(&mut self, performance: Box<AnyPerformanceDetails>, match_db: &MatchDatabase) -> Result<Uuid, InsertError> {
//     let threshold = NsDuration::from_secs_f64(MatchDatabase::ADD_TOO_CLOSE_THRESHOLD_SECONDS);
//     if let Some((close_performance, how_close)) = self
//         .find_close_performances_from_diff_match(performance.as_ref(), threshold, match_db)
//         .map_err(InsertError::MatchIsNotInDatabase)?
//         .first()
//     {
//         return Err(InsertError::TooClose(close_performance.uuid().0, *how_close));
//     }

//     if let Some(existing_performance) = self.find_performance_by_uuid(performance.uuid()) {
//         return Err(InsertError::ExistsAlready(existing_performance.uuid().0));
//     }

//     let uuid = performance.uuid();
//     self.performances.push(PerformanceEntry {
//         perf: performance,
//         cached_timestamp: Cell::new(None), // TODO: this cache thing here should not be None
//     });
//     Ok(uuid.0)
// }
