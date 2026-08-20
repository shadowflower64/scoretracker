//! Data structures for YARG (Yet Another Rhythm Game).
use crate::data::game::Game;
use crate::data::game::song::{SongAlbumInfo, SongTrait};
use crate::data::scoreboard::performance::{self, CommonPerformanceInfo, PerformanceMetadata, PerformanceTrait};
use crate::util::command_line::{AskError, ask_string, ask_u64, ask_uuid, ask_yn};
use crate::util::normalize_unsigned_to_unit_range;
use crate::util::percentage::Percentage;
use crate::util::uuid::UuidString;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A playable part in the chart.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
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
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
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
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Quickplay,
    Practice,
    PlayAShow,
    PlayAlongReplay,
    OnlineUnofficial,
}

/// A modifier (chart mutator).
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
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
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Performance {
    #[serde(flatten)]
    pub common: CommonPerformanceInfo,

    /// Timestamp of the performance - specifically, the timestamp of the first frame of the end screen. Can be approximate.
    // this should be moved to Match
    // pub timestamp: NsTimestamp,

    /// Named ID of the song.
    pub song_id: String,

    /// Played instrument.
    pub instrument: Instrument,

    /// Difficulty level of the chart.
    pub difficulty: Difficulty,

    /// Game mode that this performance was played on.
    pub mode: Mode,

    /// Amount of points at the end of the performance.
    pub score: u64,

    /// How many notes were hit successfully.
    pub notes_hit: u32,

    /// The maximum streak achieved during the performance.
    pub max_streak: u32,

    /// The amount of extra erroneous inputs.
    pub overhits: u32,

    /// Was the health meter drained, was the song failed?
    pub failed: bool,

    /// Speed of the song, as a percentage. This is not a normal `f64` to avoid rounding errors.
    pub song_speed: Percentage,

    /// List of modifiers that were used during this performance.
    pub modifiers: Vec<Modifier>,

    /// String of the game version that was played on for this performance.
    pub game_version: String,
}

#[typetag::serde(name = "yarg")]
impl PerformanceTrait for Performance {
    fn common(&self) -> &CommonPerformanceInfo {
        &self.common
    }
    fn ask_for_performance_edit(&mut self) -> Result<(), AskError> {
        self.common.comment = Some(ask_string("comment", self.comment().map(str::to_owned))?);
        Ok(())
    }
    fn sorting_key(&self) -> f64 {
        normalize_unsigned_to_unit_range(self.score) + (self.failed as u8 as f64)
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
            common: CommonPerformanceInfo {
                uuid: Uuid::now_v7().into(),
                player_uuid: ask_uuid("player uuid", None)?.into(),
                match_uuid: ask_uuid("match uuid", None)?.into(),
                proof: Vec::new(),
                // timestamp: NsTimestamp::now(),
                comment: None,
                metadata: PerformanceMetadata::new(),
            },
            song_id: ask_string("song id", None)?,
            instrument: Instrument::LeadGuitar,
            difficulty: Difficulty::Expert,
            mode: Mode::Quickplay,
            score: ask_u64("score", None)?,
            notes_hit: ask_u64("notes hit", None)? as u32,
            max_streak: ask_u64("max streak", None)? as u32,
            overhits: ask_u64("overhits", None)? as u32,
            failed: ask_yn("failed", None)?,
            song_speed: Percentage::from_percentage(ask_u64("song speed", Some(100))? as f64),
            modifiers: Vec::new(),
            game_version: ask_string("game version", Some(String::new()))?,
        }))
    }
}

// register_game!(YARG); // TODO
