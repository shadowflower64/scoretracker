use crate::data::library::database::LibraryDatabase;
use crate::data::scoreboard::MetadataValue;
use crate::data::scoreboard::performance::PerformanceDatabase;
use crate::data::scoreboard::player::PlayerDatabase;
use crate::util::file_ex::{self, FileEx};
use crate::util::filelocked::FileLockableData;
use crate::util::relative_path_from_segments;
use crate::util::timestamp::{NsDuration, NsTimestamp};
use crate::util::{command_line::AskError, uuid::UuidString};
use indexmap::IndexMap;
use relative_path::{RelativePath, RelativePathBuf};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::LazyLock;
use uuid::Uuid;

pub type MatchMetadata = IndexMap<String, MetadataValue>;
pub type SongId = String;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CommonMatchInfo {
    /// UUID of the match.
    pub uuid: UuidString,

    /// Timestamp of the match - specifically, the timestamp of the first frame of the end screen. Can be approximate.
    pub timestamp: NsTimestamp,

    /// Named ID of the song.
    pub song_id: SongId,

    /// List of library entry UUIDs that are proof of this match.
    pub proof: Vec<UuidString>,

    /// Optional user comment.
    pub comment: Option<String>,

    /// Any additional match metadata.
    pub metadata: MatchMetadata,
}

#[typetag::serde(tag = "game")]
pub trait MatchTrait: Debug {
    fn common(&self) -> &CommonMatchInfo;
    fn game_id(&self) -> &'static str {
        self.typetag_name()
    }
    fn uuid(&self) -> UuidString {
        self.common().uuid
    }
    fn timestamp(&self) -> NsTimestamp {
        self.common().timestamp
    }
    fn song_id(&self) -> &String {
        &self.common().song_id
    }
    fn proof(&self) -> &Vec<UuidString> {
        &self.common().proof
    }
    fn comment(&self) -> &Option<String> {
        &self.common().comment
    }
    fn metadata(&self) -> &MatchMetadata {
        &self.common().metadata
    }
    fn ask_for_match_edit(&mut self) -> Result<(), AskError> {
        unimplemented!()
    }
    fn sorting_key(&self) -> f64;
    fn check_vitals(
        &self,
        _player_db: &PlayerDatabase,
        _match_db: &MatchDatabase,
        _performance_db: &PerformanceDatabase,
        _library_db: &LibraryDatabase,
    ) -> Result<(), String> {
        Ok(())
    }
}

pub type AnyMatch = Box<dyn MatchTrait>;

#[derive(Debug, Default)]
pub struct MatchDatabase {
    pub matches: Vec<AnyMatch>,
}

impl MatchDatabase {
    pub const STANDARD_PATH_SEGMENTS: [&str; 2] = ["data", "matches.jsonl"];
    pub const ADD_TOO_CLOSE_THRESHOLD_SECONDS: f64 = 60.0;

    pub fn path_within_shared_repo() -> &'static RelativePath {
        static CACHE: LazyLock<RelativePathBuf> = LazyLock::new(|| relative_path_from_segments(&MatchDatabase::STANDARD_PATH_SEGMENTS));
        &CACHE
    }

    pub fn find_other_close_matches(&self, req_m: &AnyMatch, threshold: NsDuration) -> Vec<(&AnyMatch, NsDuration)> {
        let mut search_results = self
            .matches
            .iter()
            .filter_map(|m| {
                let difference = (m.timestamp() - req_m.timestamp()).abs();
                (m.uuid() != req_m.uuid() && m.game_id() == req_m.game_id() && difference <= threshold).then_some((m, difference))
            })
            .collect::<Vec<_>>();
        search_results.sort_by(|(_, a), (_, b)| a.cmp(b));
        search_results
    }

    pub fn find_match_by_uuid(&self, uuid: UuidString) -> Option<&AnyMatch> {
        self.matches.iter().find(|x| x.uuid() == uuid)
    }

    pub fn find_match_by_uuid_mut(&mut self, uuid: UuidString) -> Option<&mut AnyMatch> {
        self.matches.iter_mut().find(|x| x.uuid() == uuid)
    }

    pub fn add(&mut self, match_data: AnyMatch) -> Result<Uuid, Uuid> {
        let threshold = NsDuration::from_secs_f64(Self::ADD_TOO_CLOSE_THRESHOLD_SECONDS);
        if let Some((close_match, _how_close)) = self.find_other_close_matches(&match_data, threshold).first() {
            return Err(close_match.uuid().0);
        }

        if let Some(existing_match) = self.find_match_by_uuid(match_data.uuid()) {
            return Err(existing_match.uuid().0);
        }

        let uuid = match_data.uuid();
        self.matches.push(match_data);
        Ok(uuid.0)
    }
}

impl FileLockableData for MatchDatabase {
    fn _inner_read<F: FileEx + ?Sized>(file_ex: &F) -> file_ex::Result<Option<Self>> {
        file_ex.read_from_jsonlines().map(|x| x.map(|y| Self { matches: y }))
    }
    fn _inner_write<F: FileEx + ?Sized>(&self, file_ex: &F) -> file_ex::Result<()> {
        file_ex.write_as_jsonlines(&self.matches)
    }
}
