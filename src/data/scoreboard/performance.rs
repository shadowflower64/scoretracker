use crate::data::library::database::LibraryDatabase;
use crate::data::scoreboard::MetadataValue;
use crate::data::scoreboard::r#match::MatchDatabase;
use crate::data::scoreboard::player::PlayerDatabase;
use crate::util::file_ex::{self, FileEx};
use crate::util::filelocked::FileLockableData;
use crate::util::relative_path_from_segments;
use crate::util::timestamp::NsDuration;
use crate::util::{command_line::AskError, uuid::UuidString};
use indexmap::IndexMap;
use relative_path::{RelativePath, RelativePathBuf};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::LazyLock;
use uuid::Uuid;

// use schemars::{Schema, SchemaGenerator, json_schema};
// use std::borrow::Cow;

// #[derive(Debug, Clone, Deserialize, Serialize)]
// #[serde(transparent)]
// pub struct PerformanceMetadata(IndexMap<String, AnyValue>);

// impl PerformanceMetadata {
//     pub fn new() -> Self {
//         Self(IndexMap::new())
//     }
// }

// impl JsonSchema for PerformanceMetadata {
//     fn schema_name() -> Cow<'static, str> {
//         "PerformanceMetadata".into()
//     }
//     fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
//         json_schema!({
//             "type": "object"
//         })
//     }
// }

pub type PerformanceMetadata = IndexMap<String, MetadataValue>;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CommonPerformanceInfo {
    /// UUID of the performance.
    pub uuid: UuidString,

    /// Player UUID.
    pub player_uuid: UuidString,

    /// Match UUID.
    pub match_uuid: UuidString,

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
    fn game_id(&self) -> &'static str {
        self.typetag_name()
    }
    fn uuid(&self) -> UuidString {
        self.common().uuid
    }
    fn player_uuid(&self) -> UuidString {
        self.common().player_uuid
    }
    fn match_uuid(&self) -> UuidString {
        self.common().match_uuid
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

pub type AnyPerformance = Box<dyn PerformanceTrait>;

#[derive(Debug, Default)]
pub struct PerformanceDatabase {
    pub performances: Vec<AnyPerformance>,
}

impl PerformanceDatabase {
    pub const STANDARD_PATH_SEGMENTS: [&str; 2] = ["data", "performances.jsonl"];

    pub fn path_within_shared_repo() -> &'static RelativePath {
        static CACHE: LazyLock<RelativePathBuf> =
            LazyLock::new(|| relative_path_from_segments(&PerformanceDatabase::STANDARD_PATH_SEGMENTS));
        &CACHE
    }

    pub fn find_close_performances_from_diff_match(
        &self,
        req_p: &AnyPerformance,
        threshold: NsDuration,
        match_db: &MatchDatabase,
    ) -> Vec<(&AnyPerformance, NsDuration)> {
        let req_m = match_db.find_match_by_uuid(req_p.match_uuid()).expect("todo");
        let mut search_results: Vec<(&AnyPerformance, NsDuration)> = self
            .performances
            .iter()
            .filter_map(|p| {
                let m = match_db.find_match_by_uuid(p.match_uuid()).expect("todo");
                let difference = (m.timestamp() - req_m.timestamp()).abs();
                (m.uuid() != req_m.uuid() && p.game_id() == req_p.game_id() && difference <= threshold).then_some((p, difference))
            })
            .collect();
        search_results.sort_by(|(_, a), (_, b)| a.cmp(b));
        search_results
    }

    pub fn find_performance_by_uuid(&self, uuid: UuidString) -> Option<&AnyPerformance> {
        self.performances.iter().find(|x| x.uuid() == uuid)
    }

    pub fn find_performance_by_uuid_mut(&mut self, uuid: UuidString) -> Option<&mut AnyPerformance> {
        self.performances.iter_mut().find(|x| x.uuid() == uuid)
    }

    pub fn add(&mut self, performance: AnyPerformance, match_db: &MatchDatabase) -> Result<Uuid, Uuid> {
        let threshold = NsDuration::from_secs_f64(MatchDatabase::ADD_TOO_CLOSE_THRESHOLD_SECONDS);
        if let Some((close_performance, _how_close)) = self
            .find_close_performances_from_diff_match(&performance, threshold, match_db)
            .first()
        {
            return Err(close_performance.uuid().0);
        }

        if let Some(existing_performance) = self.find_performance_by_uuid(performance.uuid()) {
            return Err(existing_performance.uuid().0);
        }

        let uuid = performance.uuid();
        self.performances.push(performance);
        Ok(uuid.0)
    }
}

impl FileLockableData for PerformanceDatabase {
    fn _inner_read<F: FileEx + ?Sized>(file_ex: &F) -> file_ex::Result<Option<Self>> {
        file_ex.read_from_jsonlines().map(|x| x.map(|y| Self { performances: y }))
    }
    fn _inner_write<F: FileEx + ?Sized>(&self, file_ex: &F) -> file_ex::Result<()> {
        file_ex.write_as_jsonlines(&self.performances)
    }
}
