use crate::data::scoreboard::AnyValue;
use crate::util::{command_line::AskError, uuid::UuidString};
use indexmap::IndexMap;
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::Debug;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct PerformanceMetadata(IndexMap<String, AnyValue>);

impl PerformanceMetadata {
    pub fn new() -> Self {
        Self(IndexMap::new())
    }
}

impl JsonSchema for PerformanceMetadata {
    fn schema_name() -> Cow<'static, str> {
        "PerformanceMetadata".into()
    }
    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "object"
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CommonPerformanceInfo {
    /// UUID of the performance.
    pub uuid: UuidString,

    /// Player UUID.
    pub player_uuid: UuidString,

    /// List of library entry UUIDs that are proof of this performance.
    pub proof: Vec<UuidString>,

    /// Optional user comment.
    pub comment: Option<String>,

    /// Any additional performance metadata.
    pub metadata: PerformanceMetadata,
}

#[typetag::serde(tag = "game")]
pub trait PerformanceTrait: Debug {
    fn common(&self) -> &CommonPerformanceInfo;
    fn uuid(&self) -> &UuidString {
        &self.common().uuid
    }
    fn player_uuid(&self) -> &UuidString {
        &self.common().player_uuid
    }
    fn proof(&self) -> &Vec<UuidString> {
        &self.common().proof
    }
    fn comment(&self) -> &Option<String> {
        &self.common().comment
    }
    fn metadata(&self) -> &PerformanceMetadata {
        &self.common().metadata
    }
    fn ask_for_performance_edit(&mut self) -> Result<(), AskError> {
        unimplemented!()
    }
    fn sorting_key(&self) -> f64;
}

pub type AnyPerformance = Box<dyn PerformanceTrait>;
