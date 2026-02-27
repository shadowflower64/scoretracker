//! Data structures for YARG (Yet Another Rhythm Game).
use crate::game::Game;
use crate::scoreboard::performance::{self, CommonPerformanceInfo, PerformanceMetadata, PerformanceTrait};
use crate::songdb::song::{SongAlbumInfo, SongTrait};
use crate::util::cmd::{AskError, ask_string, ask_u64, ask_uuid};
use crate::util::percentage::Percentage;
use crate::util::timestamp::NsTimestamp;
use crate::util::uuid::UuidString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A playable part in the chart.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Instrument {
    LeadGuitar,
    MelodyGuitar,
    RhythmGuitar,
    BassGuitar,
    Drums4L,
    Drums5L,
    ProDrums,
    EliteDrums,
    Keys5L,
    ProKeys,
    Vocals,
    Harmony1,
    Harmony2,
    Harmony3,
}

/// Difficulty that the performance was played on.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Beginner,
    Easy,
    Medium,
    Hard,
    Expert,
    ExpertPlus,
}

/// Game mode.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Quickplay,
    Practice,
    PlayAShow,
    PlayAlongReplay,
    OnlineUnofficial,
}

/// A modifier (chart mutator).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Modifier {
    AllStrums,
    AllHopos,
    AllTaps,
    HoposToTaps,
    TapsToHopos,
    NoRangeShifts,
    NoKicks,
    NoDynamics,
}

/// A YARG performance - a performance of one player playing on one instrument on a specific chart.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Performance {
    ///// Who played what and when. /////
    /// Timestamp of the performance - specifically, the timestamp of the first frame of the end screen. Can be approximate.
    pub timestamp: NsTimestamp,

    /// Player UUID.
    pub player_uuid: UuidString,

    /// Named ID of the song.
    pub song_id: String,

    /// Played instrument.
    pub instrument: Instrument,

    /// Difficulty level of the chart.
    pub difficulty: Difficulty,

    /// Game mode that this performance was played on.
    pub mode: Mode,

    ///// Score and stats. /////
    /// Amount of points at the end of the performance.
    pub score: u64,

    /// How many notes were hit successfully.
    pub notes_hit: u64,

    /// The maximum streak achieved during the performance.
    pub max_streak: u64,

    /// The amount of extra erroneous inputs.
    pub overhits: u64,

    ///// Additional song settings. /////
    /// Speed of the song, as a percentage. This is not a normal `f64` to avoid rounding errors.
    pub song_speed: Percentage,

    /// List of modifiers that were used during this performance.
    pub modifiers: Vec<Modifier>,

    ///// Game settings and information. /////
    /// String of the game version that was played on for this performance.
    pub game_version: String,

    ///// Stuff outside of the game. /////
    /// List of library entry UUIDs that are proof of this performance.
    pub proof: Vec<UuidString>,

    /// Optional user comment.
    pub comment: Option<String>,

    /// Any additional performance metadata.
    pub metadata: PerformanceMetadata,
}

#[typetag::serde(name = "yarg")]
impl PerformanceTrait for Performance {
    fn common(&self) -> CommonPerformanceInfo {
        todo!()
    }
    fn score(&self) -> f64 {
        self.score as f64
    }
    fn proof(&self) -> Vec<UuidString> {
        self.proof.clone()
    }
    fn comment(&self) -> Option<String> {
        self.comment.clone()
    }
    fn metadata(&self) -> PerformanceMetadata {
        self.metadata.clone()
    }
    fn ask_for_performance_edit(&mut self) -> Result<(), AskError> {
        self.comment = Some(ask_string("comment", self.comment())?);
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Song {
    pub global_song_id: Option<UuidString>,
    pub title: String,
    pub artist: String,
    pub album: SongAlbumInfo,
    pub year: String,
}

impl SongTrait for Song {
    fn global_song_id(&self) -> Option<Uuid> {
        self.global_song_id.map(|x| x.0)
    }
    fn title(&self) -> String {
        self.title.clone()
    }
    fn artist(&self) -> String {
        self.artist.clone()
    }
    fn album(&self) -> Option<SongAlbumInfo> {
        Some(self.album.clone())
    }
    fn year(&self) -> Option<i64> {
        self.year.parse().ok()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct YARG;

#[typetag::serde(name = "yarg")]
impl Game for YARG {
    fn pretty_name(&self) -> &'static str {
        "Yet Another Rhythm Game"
    }
    fn url_shortname(&self) -> &'static str {
        "yarg"
    }

    fn ask_for_performance_new(&self) -> Result<Box<dyn performance::PerformanceTrait>, AskError> {
        Ok(Box::new(Performance {
            player_uuid: ask_uuid("player uuid", None)?.into(),
            song_id: ask_string("song id", None)?,
            instrument: Instrument::LeadGuitar,
            difficulty: Difficulty::Expert,
            mode: Mode::Quickplay,
            score: ask_u64("score", None)?,
            notes_hit: ask_u64("notes hit", None)?,
            max_streak: ask_u64("max streak", None)?,
            overhits: ask_u64("overhits", None)?,
            song_speed: Percentage::from_percentage(ask_u64("song speed", Some(100))? as f64),
            modifiers: Vec::new(),
            game_version: ask_string("game version", Some(String::new()))?,
            proof: Vec::new(),
            timestamp: NsTimestamp::now(),
            comment: None,
            metadata: HashMap::new(),
        }))
    }
}
