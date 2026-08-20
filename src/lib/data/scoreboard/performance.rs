use crate::data::library::database::LibraryDatabase;
use crate::data::scoreboard::MetadataValue;
use crate::data::scoreboard::r#match::MatchDatabase;
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
use std::cell::Cell;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::sync::LazyLock;
use thiserror::Error;
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
    fn proof(&self) -> &[UuidString] {
        &self.common().proof
    }
    fn comment(&self) -> Option<&str> {
        self.common().comment.as_deref()
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct PerformanceEntry {
    pub perf: AnyPerformance,
    #[serde(skip)]
    pub cached_timestamp: Cell<Option<NsTimestamp>>,
}

impl Deref for PerformanceEntry {
    type Target = AnyPerformance;
    fn deref(&self) -> &Self::Target {
        &self.perf
    }
}

impl DerefMut for PerformanceEntry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.perf
    }
}

#[derive(Debug, Default)]
pub struct PerformanceDatabase {
    pub performances: Vec<PerformanceEntry>,
}

#[derive(Debug, Error)]
pub enum InsertError {
    #[error("performance is too close to an existing performance: {0} (time difference: {1})")]
    TooClose(Uuid, NsDuration),
    #[error("performance is already in the database: {0}")]
    ExistsAlready(Uuid),
    #[error("match uuid referenced by performance is not present in the match database: {0}")]
    MatchIsNotInDatabase(Uuid),
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
        req_p: &dyn PerformanceTrait,
        threshold: NsDuration,
        match_db: &MatchDatabase,
    ) -> Result<Vec<(&dyn PerformanceTrait, NsDuration)>, Uuid> {
        let req_match_uuid = req_p.match_uuid();
        let req_m = match_db.find_match_by_uuid(req_match_uuid).ok_or(req_match_uuid)?; // TODO: optimize this away
        let req_timestamp = req_m.timestamp();
        let mut search_results: Vec<(&dyn PerformanceTrait, NsDuration)> = self
            .performances
            .iter()
            .filter_map(|ent| {
                if ent.cached_timestamp.get().is_none() {
                    ent.cached_timestamp.set({
                        let m = match_db
                            .find_match_by_uuid(ent.match_uuid())
                            .expect("todo: performance's match data not found - database is in an invalid state");
                        Some(m.timestamp())
                    });
                }
                let p_timestamp = ent.cached_timestamp.get().expect("the code above should set the value to Some");

                let difference = (p_timestamp - req_timestamp).abs();
                (ent.match_uuid() != req_match_uuid && ent.game_id() == req_p.game_id() && difference <= threshold)
                    .then_some((ent.as_ref(), difference))
            })
            .collect();
        search_results.sort_by_key(|(_, how_close)| *how_close);
        Ok(search_results)
    }

    pub fn find_performance_by_uuid(&self, uuid: UuidString) -> Option<&dyn PerformanceTrait> {
        self.performances.iter().find(|x| x.uuid() == uuid).map(|x| x.as_ref())
    }

    pub fn find_performance_by_uuid_mut(&mut self, uuid: UuidString) -> Option<&mut AnyPerformance> {
        self.performances.iter_mut().find(|x| x.uuid() == uuid).map(|x| x.deref_mut())
    }

    pub fn insert(&mut self, performance: AnyPerformance, match_db: &MatchDatabase) -> Result<Uuid, InsertError> {
        let threshold = NsDuration::from_secs_f64(MatchDatabase::ADD_TOO_CLOSE_THRESHOLD_SECONDS);
        if let Some((close_performance, how_close)) = self
            .find_close_performances_from_diff_match(performance.as_ref(), threshold, match_db)
            .map_err(InsertError::MatchIsNotInDatabase)?
            .first()
        {
            return Err(InsertError::TooClose(close_performance.uuid().0, *how_close));
        }

        if let Some(existing_performance) = self.find_performance_by_uuid(performance.uuid()) {
            return Err(InsertError::ExistsAlready(existing_performance.uuid().0));
        }

        let uuid = performance.uuid();
        self.performances.push(PerformanceEntry {
            perf: performance,
            cached_timestamp: Cell::new(None), // TODO: this cache thing here should not be None
        });
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
