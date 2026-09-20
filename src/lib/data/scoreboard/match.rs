use crate::data::library::entry::LibraryDatabase;
use crate::data::scoreboard::MetadataValue;
use crate::data::scoreboard::performance::PerformanceDatabase;
use crate::data::scoreboard::player::PlayerDatabase;
use crate::util::file_ex::{self, FileEx};
use crate::util::filelocked::FileLockableData;
use crate::util::relative_path_from_segments;
use crate::util::timestamp::{NsDuration, NsTimestamp};
use crate::util::{command_line::AskError, uuid::UuidString};
use dyn_clone::{DynClone, clone_trait_object};
use relative_path::{RelativePath, RelativePathBuf};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::LazyLock;
use thiserror::Error;
use uuid::Uuid;

pub type MatchMetadata = serde_json::Value;
pub type SongId = String;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Match {
    /// UUID of the match.
    pub match_uuid: UuidString,

    /// Timestamp of the match - specifically, the timestamp of the first frame of the end screen. Can be approximate.
    pub timestamp: NsTimestamp,

    /// Named ID of the chartset.
    pub chartset_id: SongId,

    /// List of library entry UUIDs that are proof of this match.
    pub proof: Vec<UuidString>,

    /// Game-specific details of the match.
    pub details: Box<AnyMatchDetails>,

    /// Any additional match metadata.
    pub metadata: MatchMetadata,
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
