use crate::scoreboard::AnyValue;
use crate::util::{command_line::AskError, uuid::UuidString};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};

pub type PerformanceMetadata = HashMap<String, AnyValue>;

#[derive(Debug, Clone, Deserialize, Serialize)]
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
    fn common(&self) -> CommonPerformanceInfo;
    fn uuid(&self) -> UuidString {
        self.common().uuid
    }
    fn player_uuid(&self) -> UuidString {
        self.common().player_uuid
    }
    fn proof(&self) -> Vec<UuidString> {
        self.common().proof
    }
    fn comment(&self) -> Option<String> {
        self.common().comment
    }
    fn metadata(&self) -> PerformanceMetadata {
        self.common().metadata
    }
    fn score(&self) -> f64;
    fn ask_for_performance_edit(&mut self) -> Result<(), AskError> {
        unimplemented!()
    }
}
