use crate::data::metadata::ArbitraryMetadata;
use crate::util::timestamp::NsTimestamp;
use crate::util::{command_line::AskError, uuid::UuidString};
use dyn_clone::{DynClone, clone_trait_object};
use postgres_types::FromSql;
use schemars::{JsonSchema, json_schema};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::borrow::Cow;
use std::fmt::Debug;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Match {
    /// UUID of the match.
    pub match_uuid: UuidString,

    /// Timestamp of the match.
    ///
    /// This value can be approximate.
    /// If you want to be really specific and consistent, use the timestamp of the first frame of the end screen.
    pub timestamp: NsTimestamp,

    /// Game ID.
    pub game: String,

    /// Named ID of the chartset.
    pub chartset_id: String,

    /// List of library entry UUIDs that are proof of this match.
    pub proofs: Vec<UuidString>,

    /// Game-specific details of the match.
    pub details: Box<AnyMatchDetails>,

    /// Any additional match metadata.
    pub metadata: ArbitraryMetadata,

    /// Timestamp of when this performance was added to the database.
    pub timestamp_added: NsTimestamp,
}

impl Match {
    pub fn downcast_details<T: MatchDetails>(&self) -> Option<&T> {
        self.details.downcast_ref()
    }
    pub fn from_postgres_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            match_uuid: row.try_get("match_uuid")?,
            timestamp: row.try_get("timestamp")?,
            game: row.try_get("game")?,
            chartset_id: row.try_get("chartset_id")?,
            proofs: row.try_get("proofs")?,
            details: row.try_get("details")?,
            metadata: row.try_get("metadata")?,
            timestamp_added: row.try_get("timestamp_added")?,
        })
    }
}

#[typetag::serde(tag = "game")]
pub trait MatchDetails: Debug + DynClone + Any {
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

impl dyn MatchDetails {
    fn any_ref(&self) -> &dyn Any {
        self
    }
    fn downcast_ref<T: MatchDetails>(&self) -> Option<&T> {
        self.any_ref().downcast_ref()
    }
}

clone_trait_object! {MatchDetails}
pub type AnyMatchDetails = dyn MatchDetails;

impl<'a> FromSql<'a> for Box<AnyMatchDetails> {
    fn from_sql(ty: &postgres_types::Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let value = serde_json::Value::from_sql(ty, raw)?;
        let details = serde_json::from_value(value)?;
        Ok(details)
    }

    fn accepts(ty: &postgres_types::Type) -> bool {
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

// TODO: implement warning when two very close matches are added.
const ADD_TOO_CLOSE_THRESHOLD_SECONDS: f64 = 60.0;
