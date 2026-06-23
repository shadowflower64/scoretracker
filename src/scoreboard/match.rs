use crate::scoreboard::AnyValue;
use crate::util::{cmd::AskError, timestamp::NsTimestamp, uuid::UuidString};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};
pub type MatchMetadata = HashMap<String, AnyValue>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommonMatchInfo {
    /// UUID of the match.
    pub uuid: UuidString,

    /// Timestamp of the match - specifically, the timestamp of the first frame of the end screen. Can be approximate.
    pub timestamp: NsTimestamp,

    /// Named ID of the song.
    pub song_id: String,

    /// Performances belonging to this match.
    pub performance_ids: Vec<UuidString>,

    /// List of library entry UUIDs that are proof of this match.
    pub proof: Vec<UuidString>,

    /// Optional user comment.
    pub comment: Option<String>,

    /// Any additional match metadata.
    pub metadata: MatchMetadata,
}

#[typetag::serde(tag = "game")]
pub trait MatchTrait: Debug {
    fn common(&self) -> CommonMatchInfo;
    fn uuid(&self) -> UuidString {
        self.common().uuid
    }
    fn timestamp(&self) -> NsTimestamp {
        self.common().timestamp
    }
    fn song_id(&self) -> String {
        self.common().song_id
    }
    fn performance_ids(&self) -> Vec<UuidString> {
        self.common().performance_ids
    }
    fn proof(&self) -> Vec<UuidString> {
        self.common().proof
    }
    fn comment(&self) -> Option<String> {
        self.common().comment
    }
    fn metadata(&self) -> MatchMetadata {
        self.common().metadata
    }
    fn score(&self) -> f64;
    fn ask_for_match_edit(&mut self) -> Result<(), AskError> {
        unimplemented!()
    }
}
